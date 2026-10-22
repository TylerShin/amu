import { runWindows, windowsEnvironment } from './windows-toolchain.mjs';

const env = windowsEnvironment('x64');
runWindows('cargo', ['fmt', '--all', '--check'], { env });
runWindows('cargo', ['clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings'], { env });
runWindows('cargo', ['test', '--workspace', '--locked'], { env });
runWindows(process.execPath, ['scripts/build-windows.mjs'], { env });
