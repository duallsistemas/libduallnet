"""Package an already built x64 DLL/SO, bindings, provenance and checksums."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import re
import subprocess
import tarfile
import tomllib
import zipfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target', required=True, choices=[
        'x86_64-pc-windows-msvc', 'x86_64-unknown-linux-gnu'])
    parser.add_argument('--allow-dirty', action='store_true',
                        help='package a build from uncommitted sources (never for releases)')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    package = tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf-8'))['package']
    name, version = package['name'], package['version']
    linux = args.target.endswith('linux-gnu')
    binary = root / 'target' / args.target / 'release' / (f'lib{name}.so' if linux else f'{name}.dll')
    payload = binary.read_bytes()
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
    dirty = bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=root, text=True).strip())
    if dirty and not args.allow_dirty:
        raise SystemExit('Working tree is dirty; commit first or pass --allow-dirty')
    # Sem Cargo.lock versionado, 'revision' nao identifica as dependencias empacotadas.
    if subprocess.run(['git', 'ls-files', '--error-unmatch', 'Cargo.lock'], cwd=root,
                      capture_output=True).returncode != 0:
        raise SystemExit('Cargo.lock must be committed')
    metadata = {'package': name, 'version': version, 'target': args.target,
                'revision': revision, 'dirty': dirty,
                'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip()}
    binary_name = f'lib{name}.so.{version}' if linux else f'{name}.dll'
    if linux:
        dynamic = subprocess.check_output(['readelf', '-d', str(binary)], text=True)
        soname = f'lib{name}.so.{version.split(".")[0]}'
        if not re.search(r'\(SONAME\).*\[' + re.escape(soname) + r'\]', dynamic):
            raise SystemExit('Unexpected or missing SONAME')
        metadata['soname'] = soname
        versions = subprocess.check_output(['readelf', '--version-info', str(binary)], text=True)
        glibc = re.findall(r'GLIBC_(\d+\.\d+(?:\.\d+)?)', versions)
        metadata['minimum_glibc'] = max(glibc, key=lambda value: tuple(map(int, value.split('.')))) if glibc else None
    files = {binary_name: payload, 'BUILD.json': (json.dumps(metadata, indent=2) + '\n').encode()}
    for filename in ('README.md', 'LICENSE', 'Cargo.lock'):
        files[filename] = (root / filename).read_bytes()
    for path in sorted((root / 'bindings' / 'pascal' / 'src').glob('*.pas')):
        files['bindings/pascal/src/' + path.name] = path.read_bytes()
    files['SHA256SUMS'] = ''.join(f'{hashlib.sha256(data).hexdigest()}  {filename}\n'
        for filename, data in files.items()).encode()
    output = root / 'dist'
    output.mkdir(exist_ok=True)
    archive = output / f'{name}-{version}-{args.target}.{ "tar.gz" if linux else "zip" }'
    if linux:
        with tarfile.open(archive, 'w:gz') as tar:
            for filename, data in files.items():
                info = tarfile.TarInfo(filename)
                info.size = len(data)
                info.mode = 0o755 if filename == binary_name else 0o644
                tar.addfile(info, io.BytesIO(data))
            for alias, destination in [(soname, binary_name), (f'lib{name}.so', soname)]:
                info = tarfile.TarInfo(alias)
                info.type = tarfile.SYMTYPE
                info.linkname = destination
                info.mode = 0o777
                tar.addfile(info)
    else:
        with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as zip:
            for filename, data in files.items():
                zip.writestr(filename, data)
    archive.with_name(archive.name + '.sha256').write_text(
        f'{hashlib.sha256(archive.read_bytes()).hexdigest()}  {archive.name}\n', encoding='ascii')
    print(archive)
    print(json.dumps(metadata))


if __name__ == '__main__':
    main()
