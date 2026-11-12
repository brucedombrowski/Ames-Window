#!/usr/bin/env bash
# Look for things that should not be in a public repository or a release:
# network addresses, hardware addresses, people's addresses, machine names,
# home directories and credentials.
#
#   scripts/sensitive.sh                    # every tracked file, as checked out
#   scripts/sensitive.sh --range A..B       # lines added by the commits in a range
#   scripts/sensitive.sh --history          # every line ever added, on every ref
#   scripts/sensitive.sh PATH [PATH...]     # files or directories, e.g. an
#                                           # unpacked release; binaries are read
#                                           # for the text inside them
#
# It prints a report and exits 1 if it finds anything that is not on the
# allow-list, .sensitive-allow, where every exception carries its reason.
#
# What it looks for, by pattern:
#   ipv4      a dotted address, e.g. 192.0.2.x
#   ipv6      a full address, or a compressed one with a recognisable prefix
#   mac       six pairs of hex digits, e.g. 00:00:5e:00:53:xx
#   email     any address
#   home      a named home directory: /Users/<name>, /home/<name>, C:\Users\<name>
#   host      a name ending .local, .internal, .corp, .lan or .intranet
#   key       a private key header, or a token in a well-known format
#
# What it is: a tripwire for the ordinary ways such things get in. What it is
# not: proof of absence. It reads text; it does not know a serial number, a
# person's name, a street address or a hostname without a telltale ending,
# and an address split across lines or encoded passes. --history also looks
# at who authored each commit and says, without printing it, if an address
# that is not a no-reply address is there: that is a notice, not a failure,
# because it is the committer's choice and cannot be changed without
# rewriting history.
#
# Requires: python3, git.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
exec python3 - "$root" "$@" <<'PY'
import fnmatch, os, re, subprocess, sys

root, args = sys.argv[1], sys.argv[2:]
os.chdir(root)

OCTET = r"(?:25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)"
PATTERNS = [
    ("ipv4", re.compile(rf"(?<![\w.]){OCTET}(?:\.{OCTET}){{3}}(?![\w.])")),
    ("ipv6", re.compile(r"(?<![\w:])(?:[0-9A-Fa-f]{1,4}:){7}[0-9A-Fa-f]{1,4}(?![\w:])"
                        r"|(?<![\w:])(?:fe80|f[cd][0-9a-f]{2}|2[0-9a-f]{3})(?::[0-9a-f]{1,4}){0,6}::(?:[0-9a-f]{1,4}(?::[0-9a-f]{1,4}){0,6})?(?![\w:])", re.I)),
    ("mac", re.compile(r"(?<![\w:-])(?:[0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}(?![\w:-])")),
    ("email", re.compile(r"(?<![\w.+-])[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}(?![\w-])")),
    ("home", re.compile(r"(?:/Users/|/home/|[A-Za-z]:\\+Users\\+)[A-Za-z0-9._-]+")),
    ("host", re.compile(r"(?<![\w.-])[a-z0-9][a-z0-9-]*(?:\.[a-z0-9-]+)*\.(?:local|internal|corp|lan|intranet)(?![\w-])", re.I)),
    ("key", re.compile(r"-----BEGIN (?:[A-Z]+ )?PRIVATE KEY-----|\bAKIA[0-9A-Z]{16}\b|\bgh[pousr]_[A-Za-z0-9]{36,}\b|\bxox[abprs]-[A-Za-z0-9-]{10,}\b")),
]

# The allow-list: `kind  value  # reason`, or `path  glob  # reason` to leave
# a whole file out. In a value, * stands for any run of text. No reason, no
# exception.
allowed, skipped_paths = {}, []
allow_file = ".sensitive-allow"
if os.path.exists(allow_file):
    for number, line in enumerate(open(allow_file), 1):
        line = line.rstrip("\n")
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        body, _, reason = line.partition("#")
        parts = body.split(None, 1)
        if len(parts) != 2 or not reason.strip():
            sys.exit(f"{allow_file}:{number}: expected `kind value  # reason`")
        kind, value = parts[0], parts[1].strip()
        if kind == "path":
            skipped_paths.append(value)
        else:
            allowed.setdefault(kind, []).append(value)

def is_allowed(kind, text):
    # Values are matched whole, ignoring case; * stands for any run of text.
    return any(fnmatch.fnmatchcase(text.lower(), value.lower().replace("[", "[[]"))
               for value in allowed.get(kind, []))

def skip(path):
    # A glob is tried against the whole path and against the file's name, so
    # that `sbom*.cdx.json` also leaves out the copy inside a release package.
    return any(fnmatch.fnmatch(path, glob) or fnmatch.fnmatch(os.path.basename(path), glob)
               for glob in skipped_paths)

findings = []          # (kind, where, text)
def scan_text(text, where):
    for kind, pattern in PATTERNS:
        for match in pattern.finditer(text):
            found = match.group(0)
            if not is_allowed(kind, found):
                findings.append((kind, where, found))

def strings_in(data):
    """Text inside a binary: runs of printable ASCII, and of UTF-16, which is
    how a Windows program stores much of its text."""
    for run in re.findall(rb"[\x20-\x7e]{6,}", data):
        yield run.decode("ascii")
    for run in re.findall(rb"(?:[\x20-\x7e]\x00){6,}", data):
        yield run.decode("utf-16-le")

def scan_file(path, label=None):
    label = label or path
    if skip(label):
        return
    try:
        data = open(path, "rb").read()
    except OSError as error:
        sys.exit(f"cannot read {path}: {error}")
    if b"\0" in data[:8192]:
        for text in strings_in(data):
            scan_text(text, f"{label} (binary)")
    else:
        for number, line in enumerate(data.decode("utf-8", "replace").split("\n"), 1):
            scan_text(line, f"{label}:{number}")

def git(*command):
    return subprocess.run(("git",) + command, check=True, capture_output=True).stdout

def scan_added_lines(revisions, what):
    """Every line added by the given commits, with the commit and file."""
    log = git("log", "-p", "--no-color", "--no-ext-diff", "--format=commit %H", *revisions)
    commit = path = None
    for raw in log.decode("utf-8", "replace").split("\n"):
        if raw.startswith("commit "):
            commit = raw[7:14]
        elif raw.startswith("+++ "):
            path = raw[6:] if raw.startswith("+++ b/") else None
        elif raw.startswith("+") and not raw.startswith("+++") and path and not skip(path):
            scan_text(raw[1:], f"{commit} {path}")
    return what

notice = None
scanned = ""
if args and args[0] == "--history":
    scanned = scan_added_lines(["--all"], "every line ever added, on every ref")
    addresses = set(git("log", "--all", "--format=%ae%n%ce").decode().split())
    personal = [a for a in addresses if not is_allowed("email", a)]
    if personal:
        notice = (f"{len(personal)} address(es) that are not no-reply addresses appear as the author or "
                  "committer of commits. They are public in the history. Not printed here; see `git log`.")
elif args and args[0] == "--range":
    if len(args) != 2:
        sys.exit("usage: scripts/sensitive.sh --range A..B")
    scanned = scan_added_lines([args[1]], f"lines added in {args[1]}")
elif args and args[0].startswith("-"):
    sys.exit("usage: scripts/sensitive.sh [--history | --range A..B | PATH...]")
elif args:
    count = 0
    for target in args:
        target = os.path.join(os.environ.get("OLDPWD", root), target) if not os.path.isabs(target) and not os.path.exists(target) else target
        if os.path.isdir(target):
            for folder, _, names in sorted(os.walk(target)):
                for name in sorted(names):
                    full = os.path.join(folder, name)
                    scan_file(full, os.path.relpath(full, os.path.dirname(os.path.abspath(target)))); count += 1
        elif os.path.exists(target):
            scan_file(target, os.path.basename(target)); count += 1
        else:
            sys.exit(f"no such file or directory: {target}")
    if count == 0:
        sys.exit("nothing to scan")
    scanned = f"{count} file(s) in " + ", ".join(args)
else:
    names = [n for n in git("ls-files", "-z").decode().split("\0") if n]
    for name in names:
        if os.path.isfile(name):
            scan_file(name)
    scanned = f"{len(names)} tracked files"

print(f"Scanned {scanned}.")
if notice:
    print(f"Notice: {notice}")
if not findings:
    print("Nothing found: no network or hardware address, email address, named home directory, internal host name or credential outside the allow-list.")
    sys.exit(0)

unique = {}
for kind, where, text in findings:
    unique.setdefault((kind, text), []).append(where)
print(f"Found {len(unique)} distinct item(s) in {len(findings)} place(s):\n")
print("| Kind | Found | First place | Places |")
print("|---|---|---|---|")
for (kind, text), places in sorted(unique.items()):
    shown = text if kind != "key" else text[:12] + "..."
    print(f"| {kind} | `{shown}` | {places[0]} | {len(places)} |")
print(f"\nRemove each, or add it to {allow_file} with the reason it is safe to publish.")
sys.exit(1)
PY
