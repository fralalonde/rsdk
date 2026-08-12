#!/usr/bin/env python3
"""Minimal mock of the SDKMAN v2 API for the VHS TUI demo.

Serves a fixed, tiny catalog so `rsdk tui` can run deterministically and
offline (see demo/generate.sh, which points RSDK_API_BASE_URL at this server).
"""
import http.server
import re
import sys

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 18799

TOOLS = "java,maven,gradle,scala"

DESCRIPTIONS = """\
================================================================================
Java (Eclipse Temurin)
================================================================================
A full OpenJDK distribution from Eclipse Temurin. Ships the JDK, the JRE, and
the full compiler toolchain.
$ sdk install java
---
================================================================================
Maven
================================================================================
The Apache build tool. Manages dependencies, builds, and the project lifecycle.
$ sdk install maven
---
================================================================================
Gradle
================================================================================
A fast, flexible JVM build tool with Groovy/Kotlin DSLs and incremental builds.
$ sdk install gradle
---
================================================================================
Scala
================================================================================
The Scala compiler and tooling for the JVM.
$ sdk install scala
"""

# Java uses the pipe-delimited format consumed by decode_java_versions (the
# version id sits in the 6th field, index 5).
JAVA_VERSIONS = """\
Available: Java Versions
---
21.0.6-tem | tem | linux | x64 | tgz | 21.0.6-tem | 21
17.0.9-tem | tem | linux | x64 | tgz | 17.0.9-tem | 17
11.0.22-tem | tem | linux | x64 | tgz | 11.0.22-tem | 11
8.0.402-tem | tem | linux | x64 | tgz | 8.0.402-tem | 8
25.ea.1-open | open | linux | x64 | tgz | 25.ea.1-open | 25
"""


def column_versions(versions):
    # decode_versions keeps lines between the 2nd `===` separator and the next
    # one, taking the first whitespace-separated token of each.
    return "===\nAvailable versions\n===\n" + "\n".join(versions)


VERSIONS = {
    "java": JAVA_VERSIONS,
    "maven": column_versions(["3.9.9", "3.9.8", "3.9.6"]),
    "gradle": column_versions(["8.11.1", "8.10.2", "8.9"]),
    "scala": column_versions(["3.5.1", "2.13.15", "2.12.19"]),
}


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        path = self.path.split("?")[0]
        if path == "/candidates/all":
            body = TOOLS
        elif path == "/candidates/list":
            body = DESCRIPTIONS
        else:
            m = re.match(r"/candidates/([^/]+)/[^/]+/versions/list", path)
            body = VERSIONS.get(m.group(1)) if m else None
        if body is None:
            self.send_response(404)
            self.end_headers()
            return
        data = body.encode()
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, format, *args):  # keep the demo output quiet
        pass


if __name__ == "__main__":
    print(f"mock SDKMAN API listening on 127.0.0.1:{PORT}", flush=True)
    http.server.HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
