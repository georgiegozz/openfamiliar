# Windows code signing

OpenFamiliar can be self-signed for an isolated development or QA machine, but
a self-signed installer is not suitable for public GitHub distribution. Windows
treats it like an unsigned download until each recipient explicitly trusts the
private test certificate.

## Recommended release paths

1. **Microsoft Store / MSIX**: lowest-friction path for a first public project.
   Store-distributed MSIX packages are signed by Microsoft and do not show the
   normal SmartScreen reputation warning. Packaging and Store enrollment remain
   separate release work; the current repository produces MSI and NSIS.
2. **Publicly trusted OV code-signing certificate**: appropriate for MSI/NSIS
   downloads from GitHub Releases. Buy it from a trusted certificate authority,
   keep the private key in its required protected storage, and sign both the app
   executable and installers. A new binary can still show SmartScreen until it
   earns reputation.
3. **SignPath Foundation**: potentially free for accepted open-source projects.
   It requires an application plus repository, build-provenance, security, and
   maintenance conditions; it should be evaluated after the public project and
   repeatable CI build exist.
4. **Microsoft Artifact Signing Public Trust**: cloud signing is inexpensive,
   but as of 2026-07-14 individual onboarding is listed only for the United
   States and Canada. Recheck availability before selecting it for an individual
   publisher in Mexico.

Do not buy EV solely to bypass SmartScreen. Microsoft states that OV, EV, and
Artifact Signing all need reputation for each new file; EV has not provided an
instant warning bypass since 2024.

## Local self-signed test only

Create a test certificate in the current user's certificate store:

```powershell
$testCertificate = New-SelfSignedCertificate `
  -Type CodeSigningCert `
  -Subject 'CN=OpenFamiliar Local Test' `
  -CertStoreLocation 'Cert:\CurrentUser\My'
```

Sign only a disposable local build and verify it with SignTool. Other machines
will not trust it unless the public certificate is manually installed into an
appropriate trusted store. Never commit or share its private key, PFX, password,
or thumbprint.

## Public certificate signing pattern

Install the Windows SDK SignTool and expose only non-secret configuration as
environment variables. The certificate must already be available through the
provider-supported Windows certificate store or hardware/cloud signer.

```powershell
signtool sign `
  /sha1 $env:WINDOWS_SIGNING_CERT_SHA1 `
  /fd SHA256 `
  /tr $env:WINDOWS_TIMESTAMP_URL `
  /td SHA256 `
  .\target\release\openfamiliar.exe

signtool verify /pa /v .\target\release\openfamiliar.exe
```

Repeat signing for each final MSI and NSIS installer after the application
binary is signed. Timestamping is required so a valid release remains verifiable
after certificate expiration.

The repository verifier reports file hashes and current Authenticode state:

```powershell
pnpm release:verify
pwsh -NoProfile `
  -File .\scripts\release\Test-WindowsArtifacts.ps1 `
  -RequireSignature
```

The first command allows `NotSigned` for local beta diagnosis. The second is the
release gate and fails unless every discovered application EXE, MSI, and NSIS
signature is trusted and valid.

## References

- [Microsoft: SmartScreen application reputation](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation)
- [Microsoft: code-signing options](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)
- [Microsoft: SignTool](https://learn.microsoft.com/en-us/dotnet/framework/tools/signtool-exe)
- [Microsoft Artifact Signing availability](https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart)
- [SignPath Foundation](https://signpath.org/)
