import importlib.util
import io
import json
import tarfile
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location('gate', Path(__file__).with_name('check-public-package.py'))
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class PublicPackageGateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.pkg = {'name': '@example/module', 'version': '1.0.0', 'license': 'Apache-2.0',
                    'files': ['index.js', 'LICENSE', 'NOTICE', 'README.md'],
                    'main': './index.js', 'repository': {'url': 'git+https://github.com/example/module.git'},
                    'publishConfig': {'registry': 'https://registry.npmjs.org'}}
        self.write_manifest()
        for name in ['LICENSE', 'NOTICE', 'README.md']:
            (self.root / name).write_text('Example fixture')

    def tearDown(self):
        self.temp.cleanup()

    def write_manifest(self):
        (self.root / 'package.json').write_text(json.dumps(self.pkg))

    def archive(self, extra=None, omit=None):
        entries = {'package/package.json': json.dumps(self.pkg).encode(),
                   **{f'package/{n}': b'fixture' for n in ['LICENSE', 'NOTICE', 'README.md', 'index.js']}}
        entries.update(extra or {})
        for n in omit or []:
            del entries[n]
        archive = self.root / 'example.tgz'
        with tarfile.open(archive, 'w:gz') as target:
            for name, value in entries.items():
                item = tarfile.TarInfo(name)
                if isinstance(value, tarfile.TarInfo):
                    target.addfile(value)
                else:
                    item.size = len(value)
                    target.addfile(item, io.BytesIO(value))
        return archive

    def inspect(self, archive):
        return gate.check_archive(archive, 'npm', self.pkg['name'], self.pkg['version'])

    def test_valid_manifest_and_archive(self):
        self.assertEqual(gate.check_manifest(self.root, 'npm')[0], [])
        self.assertEqual(self.inspect(self.archive()), [])

    def test_private_package_or_local_dependency_rejected(self):
        self.pkg['private'] = True
        self.pkg['dependencies'] = {'@example/peer': 'file:../peer.tgz'}
        self.write_manifest()
        failures = gate.check_manifest(self.root, 'npm')[0]
        self.assertIn('private npm package', failures)
        self.assertTrue(any('non-registry dependency' in x for x in failures))

    def test_missing_built_entry_and_license_rejected(self):
        failures = self.inspect(self.archive(omit=['package/index.js', 'package/LICENSE']))
        self.assertTrue(any('entry point' in x for x in failures))
        self.assertIn('archive missing LICENSE', failures)

    def test_private_state_and_credential_payload_rejected(self):
        failures = self.inspect(self.archive({'package/registration/item.json': b'{}',
            'package/example.txt': b'-----BEGIN' + b' PRIVATE KEY-----'}))
        self.assertTrue(any('private path' in x for x in failures))
        self.assertTrue(any('credential candidate' in x for x in failures))

    def test_traversal_and_symlink_rejected_without_extraction(self):
        link = tarfile.TarInfo('package/link')
        link.type = tarfile.SYMTYPE
        link.linkname = '../../outside'
        failures = self.inspect(self.archive({'../outside': b'bad', 'package/link': link}))
        self.assertEqual(failures.count('unsafe archive member'), 2)
        self.assertFalse((self.root.parent / 'outside').exists())

    def test_public_catalog_cannot_include_private_entries(self):
        (self.root / 'public-package-policy.json').write_text(json.dumps({'repository_ids': ['example-public']}))
        (self.root / 'catalog-v2.json').write_text(json.dumps({'entries': [{'repository_id': 'example-private'}]}))
        self.assertTrue(any('public allowlist' in x for x in gate.check_manifest(self.root, 'npm')[0]))

    def test_cargo_private_registry_and_source_replacement_rejected(self):
        (self.root / 'Cargo.toml').write_text('''[package]
name = "example-crate"
version = "1.0.0"
rust-version = "1.97"
license = "Apache-2.0"
repository = "https://github.com/example/crate"
publish = ["crates-io"]
[dependencies]
example = { version = "1.0.0", registry = "private", path = "../example" }
''')
        (self.root / '.cargo').mkdir()
        (self.root / '.cargo/config.toml').write_text('[source.crates-io]\nreplace-with = "local"\n')
        failures = gate.check_manifest(self.root, 'cargo')[0]
        self.assertTrue(any('alternate-registry dependency' in x for x in failures))
        self.assertTrue(any('local, Git' in x for x in failures))
        self.assertTrue(any('source replacement' in x for x in failures))


if __name__ == '__main__':
    unittest.main()
