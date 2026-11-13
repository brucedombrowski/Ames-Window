#!/usr/bin/env bash
# Look for things that should not be in a public repository or a release:
# network addresses, hardware addresses, people's addresses, machine names,
# home directories and credentials.
#
#   scripts/sensitive.sh                  # every tracked file, as checked out
#   scripts/sensitive.sh --range A..B     # what the commits in a range add:
#                                         # every version of every file, every
#                                         # file name and every commit message;
#                                         # one commit is X^!
#   scripts/sensitive.sh --history        # the same for every commit on every
#                                         # ref, plus annotated tag messages
#   scripts/sensitive.sh PATH [PATH...]   # files or directories, relative to
#                                         # where you are: e.g. a downloaded
#                                         # release. Zip and gzip files are
#                                         # opened; binaries are read for the
#                                         # text inside them
#
#   --show   print what was found in full. Without it, each value is masked to
#            its first two characters and its length, because CI logs and a
#            release run's summary page are public.
#
# Exit status: 0 nothing found; 1 something found that is not an exception in
# .sensitive-allow; 2 the scan could not be done (bad arguments, a bad ref, an
# unreadable file). A scan that could not be done never reports "nothing".
#
# What it looks for, by pattern:
#   ipv4    a dotted address, e.g. 192.0.2.x, including one ending a sentence
#   ipv6    a full address, or a compressed one with a recognisable prefix
#   mac     six pairs of hex digits, or Cisco's three groups of four
#   email   any address
#   home    a named home directory: /Users/<name>, /home/<name>, C:\Users\<name>
#   host    a name ending .local, .internal, .corp, .lan, .intranet or .home.arpa
#   key     a private key header; a token in a well-known format (GitHub, AWS,
#           Slack, GitLab, Anthropic, JWT); a password inside a URL
#
# What it is: a tripwire for the ordinary ways such things get in. What it is
# not: proof of absence. It reads text; it does not know a serial number, a
# person's name, a street address or a host name without a telltale ending.
# It misses an address split across lines, written in words, base64- or
# hex-encoded, or inside a compressed format other than zip and gzip (a PNG's
# compressed text, for instance); an IPv4 octet written with a leading zero; a
# MAC with no separators; a bare `password=` setting. It does not read issues,
# pull requests or Actions logs, which are public and outside the repository.
# --history notes, without printing it, if a commit's author or committer
# address is not a no-reply address: that is the committer's choice and cannot
# be changed without rewriting history, so it is a notice, not a failure.
#
# Requires: python3 (3.8 or later), git.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
CALLER_DIR="$PWD" exec python3 - "$root" "$@" <<'PY'
import fnmatch, gzip, io, os, re, subprocess, sys, zipfile

root, argv = sys.argv[1], sys.argv[2:]
caller = os.environ["CALLER_DIR"]
os.chdir(root)

def fail(message):
    print(f"scripts/sensitive.sh: {message}", file=sys.stderr)
    sys.exit(2)

USAGE = "usage: scripts/sensitive.sh [--show] [--history | --range A..B | PATH...]"

show = "--show" in argv
args = [a for a in argv if a != "--show"]

OCTET = r"(?:25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)"
PATTERNS = [
    # An octet must not run on from a digit or a dot, so a longer dotted
    # number is not read as an address; a full stop that ends a sentence is
    # fine, as long as no digit follows it.
    ("ipv4", re.compile(rf"(?<![\d.]){OCTET}(?:\.{OCTET}){{3}}(?!\d|\.\d)")),
    ("ipv6", re.compile(r"(?<![\w:])(?:[0-9A-Fa-f]{1,4}:){7}[0-9A-Fa-f]{1,4}(?![\w:])"
                        r"|(?<![\w:])(?:fe80|f[cd][0-9a-f]{2}|2[0-9a-f]{3})(?::[0-9a-f]{1,4}){0,6}::(?:[0-9a-f]{1,4}(?::[0-9a-f]{1,4}){0,6})?(?![\w:])", re.I)),
    ("mac", re.compile(r"(?<![\w:-])(?:[0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}(?![\w:-])"
                       r"|(?<![\w.])[0-9A-Fa-f]{4}\.[0-9A-Fa-f]{4}\.[0-9A-Fa-f]{4}(?![\w.])")),
    ("key", re.compile(r"-----BEGIN (?:[A-Z]+ )*PRIVATE KEY(?: BLOCK)?-----"
                       r"|\b(?:AKIA|ASIA)[0-9A-Z]{16}\b"
                       r"|\bgh[pousr]_[A-Za-z0-9]{36,}\b|\bgithub_pat_[A-Za-z0-9_]{40,}\b"
                       r"|\bglpat-[A-Za-z0-9_-]{20,}\b|\bsk-ant-[A-Za-z0-9_-]{20,}\b"
                       r"|\bxox[abprs]-[A-Za-z0-9-]{10,}\b"
                       r"|\beyJ[A-Za-z0-9_-]{8,}\.eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}"
                       r"|[a-z][a-z0-9+.-]*://[^\s/:@]+:[^\s/@]+@")),
    ("email", re.compile(r"(?<![\w.+:/-])[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}(?![\w-])")),
    ("home", re.compile(r"(?:/Users/|/home/|[A-Za-z]:\\+Users\\+)[A-Za-z0-9._-]+")),
    ("host", re.compile(r"(?<![\w.-])[a-z0-9][a-z0-9-]*(?:\.[a-z0-9-]+)*\.(?:local|internal|corp|lan|intranet|home\.arpa)(?![\w-])", re.I)),
]
KINDS = {kind for kind, _ in PATTERNS}

# ---- the allow-list ---------------------------------------------------------
# `kind  value  # reason`, or `path  glob  # reason` to leave files out. In a
# value, * is the only wildcard and stands for any run of text; matching
# ignores case. A path glob with no / is also tried against a file's own name,
# so `sbom*.cdx.json` covers the copy inside a release package. No reason, an
# unknown kind, or a value that is only *, is refused.
allow, paths, used = [], [], {}
allow_file = ".sensitive-allow"
if os.path.exists(allow_file):
    for number, line in enumerate(open(allow_file, encoding="utf-8"), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        body, _, reason = line.rstrip("\n").partition("  #")
        parts = body.split(None, 1)
        where = f"{allow_file}:{number}"
        if len(parts) != 2 or not reason.strip():
            fail(f"{where}: expected `kind  value  # reason`, with two spaces before the #")
        kind, value = parts[0], parts[1].strip()
        if kind != "path" and kind not in KINDS:
            fail(f"{where}: unknown kind {kind!r}; expected path or one of {', '.join(sorted(KINDS))}")
        if value.strip("*") == "":
            fail(f"{where}: a value of only * would allow everything")
        entry = (where, kind, value)
        used[entry] = 0
        if kind == "path":
            paths.append(entry)
        else:
            allow.append((entry, re.compile(re.escape(value).replace(r"\*", ".*") + r"\Z", re.I)))

def allowed(kind, text):
    for entry, pattern in allow:
        if entry[1] == kind and pattern.match(text):
            used[entry] += 1
            return True
    return False

def skipped_by(path):
    for entry in paths:
        glob = entry[2]
        if fnmatch.fnmatchcase(path, glob) or ("/" not in glob and fnmatch.fnmatchcase(os.path.basename(path), glob)):
            used[entry] += 1
            return entry
    return None

# ---- scanning ---------------------------------------------------------------
findings = {}      # (kind, text) -> [places]
read = skipped = 0

def scan_text(text, where):
    for kind, pattern in PATTERNS:
        for match in pattern.finditer(text):
            found = match.group(0)
            if not allowed(kind, found):
                findings.setdefault((kind, found), []).append(where)

def strings_in(data):
    """Text inside a binary: runs of printable ASCII, and of UTF-16 in either
    byte order, which is how a Windows program stores much of its text."""
    for run in re.findall(rb"[\x20-\x7e]{6,}", data):
        yield run.decode("ascii")
    for run in re.findall(rb"(?:[\x20-\x7e]\x00){6,}", data):
        yield run.decode("utf-16-le")
    for run in re.findall(rb"(?:\x00[\x20-\x7e]){6,}", data):
        yield run.decode("utf-16-be")

def scan_bytes(data, label, depth=0):
    """Scans one file's bytes, opening zip and gzip files."""
    if depth < 3 and data[:4] == b"PK\x03\x04":
        try:
            with zipfile.ZipFile(io.BytesIO(data)) as archive:
                for member in archive.infolist():
                    if not member.is_dir():
                        scan_text(member.filename, f"{label} (file name)")
                        scan_bytes(archive.read(member), f"{label}:{member.filename}", depth + 1)
            return
        except zipfile.BadZipFile:
            pass
    if depth < 3 and data[:2] == b"\x1f\x8b":
        try:
            scan_bytes(gzip.decompress(data), f"{label} (gunzipped)", depth + 1)
            return
        except OSError:
            pass
    if b"\0" in data:
        for text in strings_in(data):
            scan_text(text, f"{label} (binary)")
    else:
        for number, line in enumerate(data.decode("utf-8", "replace").split("\n"), 1):
            scan_text(line, f"{label}:{number}")

def scan_path(full, label):
    global read, skipped
    if skipped_by(label):
        skipped += 1
        return
    scan_text(label, f"{label} (file name)")
    if os.path.islink(full):
        scan_text(os.readlink(full), f"{label} (link target)")
        read += 1
        return
    try:
        with open(full, "rb") as handle:
            data = handle.read()
    except OSError as error:
        fail(f"cannot read {label}: {error.strerror}")
    scan_bytes(data, label)
    read += 1

def git(*command):
    result = subprocess.run(("git",) + command, capture_output=True)
    if result.returncode != 0:
        message = result.stderr.decode("utf-8", "replace").strip().splitlines()
        fail(f"git {' '.join(command[:2])}: {message[0] if message else 'failed'}")
    return result.stdout

def scan_objects(revisions, with_tags):
    """Every version of every file reachable from the revisions and not from
    their exclusions, every path they were stored under, and every commit
    message; with_tags adds annotated tag messages. Reading whole objects,
    not patches, means a merge's own changes and a binary added and later
    removed are both read."""
    global read, skipped
    listing = git("rev-list", "--objects", *revisions).decode("utf-8", "replace").splitlines()
    first_path = {}
    for line in listing:
        sha, _, path = line.partition(" ")
        if path and sha not in first_path:
            first_path[sha] = path
    for path in sorted(set(first_path.values())):
        if not skipped_by(path):
            scan_text(path, f"{path} (file name)")
    if first_path:
        batch = subprocess.run(["git", "cat-file", "--batch"], input="\n".join(first_path).encode() + b"\n",
                               capture_output=True, check=True).stdout
        at = 0
        while at < len(batch):
            end = batch.index(b"\n", at)
            sha, kind, size = batch[at:end].decode().split()
            body = batch[end + 1:end + 1 + int(size)]
            at = end + 2 + int(size)
            if kind != "blob":
                continue
            path = first_path[sha]
            if skipped_by(path):
                skipped += 1
                continue
            scan_bytes(body, f"{path} @ {sha[:7]}")
            read += 1
    log = git("log", "--format=%h%x00%B%x01", *revisions).decode("utf-8", "replace")
    commits = [c for c in log.split("\x01") if c.strip()]
    for record in commits:
        short, _, message = record.strip("\n").partition("\x00")
        scan_text(message, f"commit {short} (message)")
    if with_tags:
        for record in git("for-each-ref", "refs/tags", "--format=%(refname:short)%00%(contents)%01").decode("utf-8", "replace").split("\x01"):
            if record.strip():
                name, _, message = record.strip("\n").partition("\x00")
                scan_text(message, f"tag {name} (message)")
    return len(commits)

notice = None
if args and args[0] == "--history":
    if len(args) != 1:
        fail(USAGE)
    count = scan_objects(["--all"], True)
    scanned = f"every version of {read} file version(s) and {count} commit message(s) on every ref, and tag messages"
    addresses = set(git("log", "--all", "--format=%ae%n%ce").decode().split())
    personal = [a for a in addresses if not allowed("email", a)]
    if personal:
        notice = (f"{len(personal)} address(es) that are not no-reply addresses appear as the author or "
                  "committer of commits. They are public in the history. Not printed here; see `git log`.")
elif args and args[0] == "--range":
    if len(args) != 2 or args[1].startswith("-"):
        fail(USAGE)
    count = scan_objects([args[1]], False)
    scanned = f"{read} file version(s) and {count} commit message(s) in {args[1]}"
    if count == 0:
        scanned += " (the range is empty)"
elif args and args[0].startswith("-"):
    fail(USAGE)
elif args:
    for target in args:
        full = target if os.path.isabs(target) else os.path.join(caller, target)
        if not os.path.lexists(full):
            fail(f"no such file or directory: {target}")
        if os.path.isdir(full) and not os.path.islink(full):
            base = os.path.dirname(os.path.abspath(full).rstrip("/"))
            for folder, folders, names in os.walk(full):
                if ".git" in folders:
                    folders.remove(".git")
                folders.sort()
                for name in sorted(names):
                    here = os.path.join(folder, name)
                    scan_path(here, os.path.relpath(here, base))
        else:
            scan_path(full, os.path.basename(full))
    if read + skipped == 0:
        fail("nothing to scan: the directories given hold no files")
    scanned = f"{read} file(s) in " + ", ".join(args)
else:
    listing = git("ls-files", "-s", "-z").decode("utf-8", "replace").split("\0")
    for record in filter(None, listing):
        meta, _, name = record.partition("\t")
        if meta.startswith("160000"):          # a submodule: not ours to read
            continue
        if os.path.lexists(name):
            scan_path(name, name)
    scanned = f"{read} tracked file(s)"

# ---- the report -------------------------------------------------------------
def masked(kind, text):
    if kind == "key":
        return text[:10] + "..." if show else f"{kind} ({len(text)} chars)"
    return text if show else f"{text[:2]}... ({len(text)} chars)"

print(f"Scanned {scanned}." + (f" Left out by .sensitive-allow: {skipped}." if skipped else ""))
if notice:
    print(f"Notice: {notice}")
uses = [(entry, n) for entry, n in used.items() if n]
if uses:
    print("Exceptions applied: " + "; ".join(f"{e[0]} ({e[1]} {e[2]}) x{n}" for e, n in uses) + ".")
if not findings:
    print("Nothing found: no network or hardware address, email address, named home directory, internal host name or credential outside the allow-list.")
    sys.exit(0)
places = sum(len(p) for p in findings.values())
print(f"Found {len(findings)} distinct item(s) in {places} place(s):\n")
print("| Kind | Found | First place | Places |")
print("|---|---|---|---|")
for (kind, text), where in sorted(findings.items()):
    print(f"| {kind} | `{masked(kind, text)}` | {where[0]} | {len(where)} |")
print(f"\nRemove each, or add it to {allow_file} with the reason it is safe to publish."
      + ("" if show else " Run with --show, locally, to see the values."))
sys.exit(1)
PY
