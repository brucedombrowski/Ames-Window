Ames Window for Windows
=======================

Version @VERSION@, built from commit @COMMIT@.

A flat trapezoid, painted like a six-pane window, spins full circle but
appears to swing back and forth. A blue pen through its centre seems to bend
and pass through solid wood. This is the Ames window illusion.


To run it
---------

Double-click ames-window.exe. There is nothing to install. It needs 64-bit
Windows 10 or 11 and nothing else: no runtime, no redistributable.

The first time, Windows may say "Windows protected your PC". That is
SmartScreen warning that the program is not signed by a publisher it knows.
It is not signed: a signing certificate costs money and this is a
demonstration. If you got this folder from the project's own Releases page
and the checksum matches (see below), choose "More info", then "Run anyway".
If your organisation forbids running unsigned programs, use the browser
version instead: https://brucedombrowski.github.io/Ames-Window/


Controls
--------

  Click             show or hide the dial that reveals the true rotation
  Drag left/right   turn the window by hand
  Space             show or hide the dial
  X                 show or hide the pen
  R                 reverse the spin

On the dial, the pale bar with a dot is the window as it really is and the
blue line is the pen, fixed to it. The amber bar is what your eye believes.


What it does and does not do
----------------------------

It opens one window and draws in it. It has no network code and makes no
connections. It reads no files and writes none: no settings, no logs, no
registry entries. To remove it, delete this folder.


What is in this folder
----------------------

  ames-window.exe           the program
  sbom-windows.cdx.json     Software Bill of Materials (CycloneDX 1.6): every
                            third-party component Cargo resolves for the
                            Windows build, with versions, licences and
                            checksums
  README.txt                this file
  LICENSE-MIT, LICENSE-APACHE
                            the program is licensed under either, at your
                            option


To check it is the file that was released
-----------------------------------------

The release page lists a SHA-256 checksum for the zip this folder came in,
in the file SHA256SUMS. In PowerShell:

  Get-FileHash ames-window-windows-x64.zip -Algorithm SHA256

The two must match. The release also carries a signed build provenance
attestation, which ties the zip to the commit and the build that made it:

  gh attestation verify ames-window-windows-x64.zip --repo brucedombrowski/Ames-Window

Before release the folder's contents were scanned with ClamAV. A clean scan
means no match against the signatures known that day, not a guarantee.


Source, issues and the rest of the documentation
------------------------------------------------

  https://github.com/brucedombrowski/Ames-Window
