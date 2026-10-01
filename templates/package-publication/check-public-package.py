#!/usr/bin/env python3
"""Check public manifests and the exact archive selected for publication."""
import argparse
import fnmatch
import json
import re
import tarfile
import tomllib
from pathlib import Path, PurePosixPath


def check_manifest(root, kind):
    root = Path(root).resolve()
    failures = []
    policy_path = root / 'public-package-policy.json'
    if policy_path.is_file():
        policy = json.loads(policy_path.read_text())
        expected = sorted(policy['repository_ids'])
        for catalog_path in [root / 'catalog-v2.json', root / 'public/catalog/v2/index.json']:
            if catalog_path.is_file():
                catalog = json.loads(catalog_path.read_text())
                if sorted(e['repository_id'] for e in catalog['entries']) != expected:
                    failures.append('catalog contains entries outside the reviewed public allowlist')
    if kind == 'npm':
        data = json.loads((root / 'package.json').read_text())
        if data.get('private') is True:
            failures.append('private npm package')
        if data.get('publishConfig', {}).get('registry') != 'https://registry.npmjs.org':
            failures.append('explicit official npm registry required')
        if not data.get('files'):
            failures.append('npm files allowlist required')
        for section in ['dependencies', 'devDependencies', 'optionalDependencies', 'peerDependencies']:
            for name, value in data.get(section, {}).items():
                if not isinstance(value, str) or value.startswith(('file:', 'link:', 'workspace:', 'git', 'http:', 'https:', '/', '.')):
                    failures.append(f'non-registry dependency: {section}/{name}')
        repository = data.get('repository', {})
        url = repository if isinstance(repository, str) else repository.get('url', '')
        if not re.fullmatch(r'git\+https://github\.com/[\w.-]+/[\w.-]+\.git', url):
            failures.append('canonical GitHub repository URL required')
        package = data
    else:
        data = tomllib.loads((root / 'Cargo.toml').read_text())
        package = data.get('package', {})
        if package.get('publish') != ['crates-io']:
            failures.append('publish = ["crates-io"] required')
        if not package.get('repository', '').startswith('https://github.com/'):
            failures.append('GitHub repository URL required')
        if not re.fullmatch(r'\d+\.\d+(?:\.\d+)?', str(package.get('rust-version', ''))):
            failures.append('explicit numeric Rust toolchain required')
        sections = [data, data.get('workspace', {})] + list(data.get('target', {}).values())
        for section in sections:
            for field in ['dependencies', 'dev-dependencies', 'build-dependencies']:
                for name, value in section.get(field, {}).items():
                    if isinstance(value, dict) and any(k in value for k in ['path', 'git', 'workspace']):
                        failures.append(f'local, Git, or inherited dependency: {field}/{name}')
                    if isinstance(value, dict) and value.get('registry', 'crates-io') != 'crates-io':
                        failures.append(f'alternate-registry dependency: {field}/{name}')
        if data.get('patch') or data.get('replace'):
            failures.append('dependency replacement must be removed for release validation')
        config = root / '.cargo/config.toml'
        if config.is_file():
            cfg = tomllib.loads(config.read_text())
            if cfg.get('source') or cfg.get('registries'):
                failures.append('source replacement or alternate registry configuration')
    if not package.get('name') or not re.fullmatch(r'\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?', str(package.get('version', ''))):
        failures.append('explicit package name and release version required')
    if not package.get('license') and not package.get('license-file'):
        failures.append('license metadata required')
    for name in ['LICENSE', 'NOTICE', 'README.md']:
        if not (root / name).is_file():
            failures.append(f'missing {name}')
    return failures, package


def check_archive(path, kind, expected_name, expected_version):
    failures = []
    with tarfile.open(path, 'r:gz') as archive:
        members = archive.getmembers()
        if len(members) > 15000 or sum(m.size for m in members) > 100_000_000:
            return ['archive exceeds review limits']
        prefix = 'package' if kind == 'npm' else f'{expected_name}-{expected_version}'
        paths = set()
        for member in members:
            parts = PurePosixPath(member.name).parts
            if not parts or parts[0] != prefix or '..' in parts or member.issym() or member.islnk() or not (member.isfile() or member.isdir()):
                failures.append('unsafe archive member')
                continue
            relative = '/'.join(parts[1:])
            if relative in paths and member.isfile():
                failures.append(f'duplicate file: {relative}')
            paths.add(relative)
            if any(x in {'registration', 'state', '.state', '.data', 'secrets', 'credentials', '.git', 'node_modules', 'target', '.output', '.nuxt'} for x in parts):
                failures.append(f'forbidden generated or private path: {relative}')
            leaf = parts[-1]
            if leaf.startswith('.env') or leaf.endswith(('.tgz', '.crate', '.p12', '.pfx', '.pem', '.key', '.sqlite', '.db')):
                failures.append(f'forbidden archive, credential, or runtime file: {relative}')
            if member.isfile():
                content = archive.extractfile(member).read()
                if re.search(rb'-----BEGIN (?:[A-Z ]*PRIVATE KEY|CERTIFICATE)-----|\bgh[pousr]_[A-Za-z0-9]{30,}', content):
                    failures.append(f'key, certificate, or credential candidate: {relative}')
        for name in ['LICENSE', 'NOTICE', 'README.md']:
            if name not in paths:
                failures.append(f'archive missing {name}')
        manifest = 'package.json' if kind == 'npm' else 'Cargo.toml'
        target = f'{prefix}/{manifest}'
        if manifest not in paths:
            failures.append('archive manifest missing')
        else:
            content = archive.extractfile(target).read().decode()
            package = json.loads(content) if kind == 'npm' else tomllib.loads(content)['package']
            if (package.get('name'), package.get('version')) != (expected_name, expected_version):
                failures.append('archive identity differs from release manifest')
            if kind == 'npm':
                def entries(value):
                    if isinstance(value, str):
                        return [value]
                    if isinstance(value, dict):
                        return [x for v in value.values() for x in entries(v)]
                    if isinstance(value, list):
                        return [x for v in value for x in entries(v)]
                    return []
                for field in ['main', 'module', 'types', 'typings', 'exports', 'bin']:
                    for target in entries(package.get(field)):
                        if not any(fnmatch.fnmatch(name, target.removeprefix('./')) for name in paths):
                            failures.append(f'archive missing npm entry point: {field}/{target}')
    return failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('kind', choices=['npm', 'cargo'])
    parser.add_argument('--root', default='.')
    parser.add_argument('--archive')
    args = parser.parse_args()
    failures, package = check_manifest(args.root, args.kind)
    if args.archive:
        failures += check_archive(args.archive, args.kind, package['name'], package['version'])
        with tarfile.open(args.archive, 'r:gz') as archive:
            names = {'/'.join(PurePosixPath(m.name).parts[1:]) for m in archive.getmembers()}
        for local in Path(args.root).iterdir():
            if local.is_file() and (local.name.startswith('LICENSE') or local.name in ['THIRD_PARTY_NOTICES.md', 'THIRD-PARTY-NOTICES.md']):
                if local.name not in names:
                    failures.append(f'archive omits applicable license or notice: {local.name}')
    print(json.dumps({'passed': not failures, 'failures': failures}, indent=2))
    raise SystemExit(bool(failures))


if __name__ == '__main__':
    main()
