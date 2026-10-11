"""Verify curl aborts an unknown-length response before full disk storage."""
import http.server
from pathlib import Path
import subprocess
import tempfile
import threading

LIMIT = 2 * 1024 * 1024


class Response(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        self.send_response(200)
        self.send_header("Transfer-Encoding", "chunked")
        self.end_headers()
        chunk = b"x" * 65536
        try:
            for _ in range(512):
                self.wfile.write(b"10000\r\n" + chunk + b"\r\n")
                self.wfile.flush()
            self.wfile.write(b"0\r\n\r\n")
        except (BrokenPipeError, ConnectionResetError):
            pass


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Response)
thread = threading.Thread(target=server.serve_forever, daemon=True)
thread.start()
try:
    with tempfile.TemporaryDirectory() as directory:
        output = Path(directory) / "response"
        result = subprocess.run(["curl", "--silent", "--max-time", "5", "--max-filesize", str(LIMIT), "--output", str(output), f"http://127.0.0.1:{server.server_port}/"], timeout=7, check=False)
        assert result.returncode == 63, "curl must reject the transfer limit"
        size = output.stat().st_size
        assert size <= LIMIT, "oversized transfer must not be fully saved"
        print(f"Unknown-length 32 MiB response rejected; saved {size} bytes (limit {LIMIT})")
finally:
    server.shutdown()
    server.server_close()
    thread.join(timeout=2)
