import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import * as https from 'https';
import { execSync } from 'child_process';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Trace
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext) {
    const outputChannel = vscode.window.createOutputChannel('HLSL Extended');
    outputChannel.appendLine('[HLSL Extended] Activating extension...');

    context.subscriptions.push(
        vscode.commands.registerCommand('hlsl-extended.restartServer', async () => {
            outputChannel.appendLine('[HLSL Extended] Restarting Language Server...');
            if (client) {
                await client.stop();
                client = undefined;
            }
            await startLanguageServer(context, outputChannel);
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('hlsl-extended.downloadServer', async () => {
            try {
                await downloadHlslValidator(context, outputChannel, true);
                vscode.window.showInformationMessage('HLSL Validator language server binary updated successfully.');
                vscode.commands.executeCommand('hlsl-extended.restartServer');
            } catch (err: any) {
                vscode.window.showErrorMessage(`Failed to update HLSL Validator: ${err?.message || err}`);
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('hlsl-extended.downloadDxc', async () => {
            try {
                await downloadDxc(context, outputChannel);
                vscode.window.showInformationMessage('DirectX Shader Compiler (DXC) downloaded successfully.');
                vscode.commands.executeCommand('hlsl-extended.restartServer');
            } catch (err: any) {
                vscode.window.showErrorMessage(`Failed to download DXC: ${err?.message || err}`);
            }
        })
    );

    await startLanguageServer(context, outputChannel);
}

export async function deactivate(): Promise<void> {
    if (client) {
        await client.stop();
        client = undefined;
    }
}

async function startLanguageServer(context: vscode.ExtensionContext, outputChannel: vscode.OutputChannel) {
    try {
        const validatorPath = await resolveHlslValidator(context, outputChannel);
        if (!validatorPath) {
            outputChannel.appendLine('[HLSL Extended] Language server executable not available. LSP disabled.');
            return;
        }

        const config = vscode.workspace.getConfiguration('hlslExtended');
        const configuredDxc = config.get<string>('dxcPath')?.trim();
        const dxcPath = (configuredDxc && fs.existsSync(configuredDxc))
            ? configuredDxc
            : await resolveDxc(context, outputChannel);

        outputChannel.appendLine(`[HLSL Extended] Using hlsl_validator: ${validatorPath}`);
        if (dxcPath) {
            outputChannel.appendLine(`[HLSL Extended] Using DXC: ${dxcPath}`);
        }

        const env: Record<string, string> = { ...process.env as Record<string, string> };
        if (dxcPath) {
            env['DXC_PATH'] = dxcPath;
        }

        const cwd = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        const serverOptions: ServerOptions = {
            run: {
                command: validatorPath,
                args: [],
                options: { env, cwd }
            },
            debug: {
                command: validatorPath,
                args: [],
                options: { env, cwd }
            }
        };

        const targetEngine = config.get<string>('targetEngine', 'auto');

        const clientOptions: LanguageClientOptions = {
            documentSelector: [
                { scheme: 'file', language: 'hlsl' },
                { scheme: 'file', language: 'shaderlab' }
            ],
            synchronize: {
                fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{hlsl,hlsli,fx,usf,ush,compute,shader,cginc}')
            },
            initializationOptions: {
                dxc_path: dxcPath,
                target_engine: targetEngine
            },
            outputChannel
        };

        client = new LanguageClient(
            'hlsl_validator',
            'HLSL Validator Language Server',
            serverOptions,
            clientOptions
        );

        const traceSetting = config.get<string>('trace.server', 'off');
        if (traceSetting === 'verbose') {
            client.setTrace(Trace.Verbose);
        } else if (traceSetting === 'messages') {
            client.setTrace(Trace.Messages);
        }

        await client.start();
        outputChannel.appendLine('[HLSL Extended] Language Server successfully started.');
    } catch (err: any) {
        outputChannel.appendLine(`[HLSL Extended] Failed to start Language Server: ${err?.message || err}`);
        vscode.window.showErrorMessage(`HLSL Extended LSP failed to start: ${err?.message || err}`);
    }
}

async function resolveHlslValidator(
    context: vscode.ExtensionContext,
    outputChannel: vscode.OutputChannel
): Promise<string | undefined> {
    const exeName = process.platform === 'win32' ? 'hlsl_validator.exe' : 'hlsl_validator';

    // 0. Check bundled binary
    const bundledBin = path.join(context.extensionPath, 'bin', exeName);
    if (fs.existsSync(bundledBin) && fs.statSync(bundledBin).isFile()) {
        outputChannel.appendLine(`[HLSL Extended] Using bundled binary: ${bundledBin}`);
        return bundledBin;
    }

    // 1. Check user explicit setting
    const config = vscode.workspace.getConfiguration('hlslExtended');
    const configured = config.get<string>('validatorPath')?.trim();
    if (configured && fs.existsSync(configured) && fs.statSync(configured).isFile()) {
        return configured;
    }

    // 2. Check PATH
    const inPath = findInPath(exeName);
    if (inPath) {
        return inPath;
    }

    // 3. Check local extension storage
    const storageDir = context.globalStorageUri.fsPath;
    const cachedBin = path.join(storageDir, 'bin', exeName);
    const cachedRoot = path.join(storageDir, exeName);

    if (fs.existsSync(cachedBin) && fs.statSync(cachedBin).isFile()) {
        return cachedBin;
    }
    if (fs.existsSync(cachedRoot) && fs.statSync(cachedRoot).isFile()) {
        return cachedRoot;
    }

    // 4. Download from GitHub Releases
    return await downloadHlslValidator(context, outputChannel, false);
}

function findFileRecursive(dir: string, targetName: string): string | undefined {
    if (!fs.existsSync(dir)) return undefined;
    try {
        const entries = fs.readdirSync(dir, { withFileTypes: true });
        for (const entry of entries) {
            const fullPath = path.join(dir, entry.name);
            if (entry.isFile()) {
                if (entry.name.toLowerCase() === targetName.toLowerCase()) {
                    return fullPath;
                }
            } else if (entry.isDirectory()) {
                const found = findFileRecursive(fullPath, targetName);
                if (found) return found;
            }
        }
    } catch {
        // ignore inaccessible directories
    }
    return undefined;
}

async function resolveDxc(
    context: vscode.ExtensionContext,
    outputChannel: vscode.OutputChannel
): Promise<string | undefined> {
    // 1. User setting
    const config = vscode.workspace.getConfiguration('hlslExtended');
    const configured = config.get<string>('dxcPath')?.trim();
    if (configured && fs.existsSync(configured) && fs.statSync(configured).isFile()) {
        return configured;
    }

    // 2. PATH
    const exe = process.platform === 'win32' ? '.exe' : '';
    const inPath = findInPath(`dxc${exe}`);
    if (inPath) {
        return inPath;
    }

    // 3. Windows standard locations (Windows SDK)
    if (process.platform === 'win32') {
        const sdkDirs = [
            'C:\\Program Files (x86)\\Windows Kits\\10\\bin',
            'C:\\Program Files\\Windows Kits\\10\\bin'
        ];
        for (const base of sdkDirs) {
            if (fs.existsSync(base)) {
                try {
                    const entries = fs.readdirSync(base);
                    for (const entry of entries) {
                        const candidate = path.join(base, entry, 'x64', 'dxc.exe');
                        if (fs.existsSync(candidate)) {
                            return candidate;
                        }
                    }
                } catch {}
            }
        }
    }

    // 4. Check local extension storage (cached download)
    const storageDir = context.globalStorageUri.fsPath;
    const dxcDir = path.join(storageDir, 'dxc');
    const exeName = `dxc${exe}`;

    const cached1 = findFileRecursive(dxcDir, exeName);
    if (cached1 && fs.existsSync(cached1) && fs.statSync(cached1).isFile()) {
        return cached1;
    }
    const cached2 = findFileRecursive(storageDir, exeName);
    if (cached2 && fs.existsSync(cached2) && fs.statSync(cached2).isFile()) {
        return cached2;
    }

    // 5. Download from microsoft/DirectXShaderCompiler GitHub Releases (Windows & Linux only)
    if (process.platform === 'darwin') {
        outputChannel.appendLine('[HLSL Extended] Microsoft DXC does not distribute official macOS binaries.');
        return undefined;
    }

    try {
        return await downloadDxc(context, outputChannel);
    } catch (err: any) {
        outputChannel.appendLine(`[HLSL Extended] Auto-download of DXC failed: ${err?.message || err}`);
        return undefined;
    }
}

async function downloadDxc(
    context: vscode.ExtensionContext,
    outputChannel: vscode.OutputChannel
): Promise<string> {
    const storageDir = context.globalStorageUri.fsPath;
    const dxcDir = path.join(storageDir, 'dxc');
    if (!fs.existsSync(dxcDir)) {
        fs.mkdirSync(dxcDir, { recursive: true });
    }

    const platform = process.platform;
    const arch = process.arch;

    return await vscode.window.withProgress(
        {
            location: vscode.ProgressLocation.Notification,
            title: 'HLSL Extended: Downloading DirectX Shader Compiler (DXC)',
            cancellable: false
        },
        async (progress) => {
            progress.report({ message: 'Checking Microsoft DXC releases...' });
            outputChannel.appendLine('[HLSL Extended] Fetching latest DXC release from GitHub...');

            const releaseUrl = 'https://api.github.com/repos/microsoft/DirectXShaderCompiler/releases/latest';
            const releaseData = await httpGetJson(releaseUrl);

            const isWindows = platform === 'win32';
            const asset = releaseData.assets?.find((a: any) => {
                const name = a.name || '';
                if (name.includes('pdb')) return false;
                if (isWindows) {
                    return name.startsWith('dxc_') && name.endsWith('.zip');
                } else if (platform === 'linux') {
                    return name.startsWith('linux_dxc_') && (name.endsWith('.tar.gz') || name.endsWith('.tgz'));
                }
                return false;
            });

            if (!asset || !asset.browser_download_url) {
                throw new Error(`No compatible DXC release asset found for ${platform}-${arch} in release ${releaseData.tag_name || 'latest'}`);
            }

            const downloadUrl = asset.browser_download_url;
            const archivePath = path.join(storageDir, asset.name);

            progress.report({ message: `Downloading ${asset.name}...` });
            outputChannel.appendLine(`[HLSL Extended] Downloading ${downloadUrl} to ${archivePath}`);
            await downloadFile(downloadUrl, archivePath);

            progress.report({ message: 'Extracting DXC compiler...' });
            outputChannel.appendLine(`[HLSL Extended] Extracting ${archivePath} to ${dxcDir}`);
            extractArchive(archivePath, dxcDir);

            // Clean up archive
            try { fs.unlinkSync(archivePath); } catch {}

            const exe = isWindows ? '.exe' : '';
            const exeName = `dxc${exe}`;

            let targetPath: string | undefined;
            if (isWindows) {
                const archFolder = arch === 'arm64' ? 'arm64' : 'x64';
                const preferred = path.join(dxcDir, 'bin', archFolder, exeName);
                if (fs.existsSync(preferred) && fs.statSync(preferred).isFile()) {
                    targetPath = preferred;
                }
            } else {
                const preferred = path.join(dxcDir, 'bin', exeName);
                if (fs.existsSync(preferred) && fs.statSync(preferred).isFile()) {
                    targetPath = preferred;
                }
            }

            if (!targetPath) {
                targetPath = findFileRecursive(dxcDir, exeName);
            }

            if (!targetPath) {
                throw new Error(`Extracted DXC executable not found in ${dxcDir}`);
            }

            if (!isWindows) {
                fs.chmodSync(targetPath, 0o755);
            }

            outputChannel.appendLine(`[HLSL Extended] DXC installed successfully at ${targetPath}`);
            return targetPath;
        }
    );
}

function findInPath(name: string): string | undefined {
    const pathEnv = process.env['PATH'] || '';
    const pathSep = process.platform === 'win32' ? ';' : ':';
    const dirs = pathEnv.split(pathSep);

    for (const dir of dirs) {
        if (!dir.trim()) continue;
        const candidate = path.join(dir.trim(), name);
        try {
            if (fs.existsSync(candidate) && fs.statSync(candidate).isFile()) {
                return candidate;
            }
        } catch {}
    }
    return undefined;
}

async function downloadHlslValidator(
    context: vscode.ExtensionContext,
    outputChannel: vscode.OutputChannel,
    force: boolean
): Promise<string> {
    const storageDir = context.globalStorageUri.fsPath;
    if (!fs.existsSync(storageDir)) {
        fs.mkdirSync(storageDir, { recursive: true });
    }

    const platform = process.platform;
    const arch = process.arch;

    let assetName = '';
    if (platform === 'win32') {
        assetName = 'hlsl_validator-x86_64-windows.zip';
    } else if (platform === 'linux') {
        assetName = 'hlsl_validator-x86_64-linux.tar.gz';
    } else {
        throw new Error(`Platform '${platform}' does not have official DXC support. Language server features remain active without DXC.`);
    }

    return await vscode.window.withProgress(
        {
            location: vscode.ProgressLocation.Notification,
            title: 'HLSL Extended: Downloading Language Server',
            cancellable: false
        },
        async (progress) => {
            progress.report({ message: 'Checking GitHub releases...' });

            const releaseUrl = 'https://api.github.com/repos/zyr1on/hlsl-shaderlab-extended/releases/latest';
            const releaseData = await httpGetJson(releaseUrl);

            const asset = releaseData.assets?.find((a: any) => a.name === assetName);
            if (!asset || !asset.browser_download_url) {
                throw new Error(`Release asset '${assetName}' not found in release ${releaseData.tag_name || 'latest'}`);
            }

            const downloadUrl = asset.browser_download_url;
            const archivePath = path.join(storageDir, assetName);

            progress.report({ message: `Downloading ${assetName}...` });
            outputChannel.appendLine(`[HLSL Extended] Downloading ${downloadUrl} to ${archivePath}`);
            await downloadFile(downloadUrl, archivePath);

            progress.report({ message: 'Extracting language server binary...' });
            outputChannel.appendLine(`[HLSL Extended] Extracting ${archivePath}`);
            extractArchive(archivePath, storageDir);

            // Clean up archive
            try { fs.unlinkSync(archivePath); } catch {}

            const exeName = platform === 'win32' ? 'hlsl_validator.exe' : 'hlsl_validator';
            const candidateRoot = path.join(storageDir, exeName);
            const candidateBin = path.join(storageDir, 'bin', exeName);

            let targetPath = '';
            if (fs.existsSync(candidateRoot)) {
                targetPath = candidateRoot;
            } else if (fs.existsSync(candidateBin)) {
                targetPath = candidateBin;
            } else {
                throw new Error(`Extracted executable not found in ${storageDir}`);
            }

            if (platform !== 'win32') {
                fs.chmodSync(targetPath, 0o755);
            }

            outputChannel.appendLine(`[HLSL Extended] Binary successfully installed at ${targetPath}`);
            return targetPath;
        }
    );
}

function extractArchive(archivePath: string, destDir: string): void {
    if (archivePath.endsWith('.zip')) {
        if (process.platform === 'win32') {
            execSync(`powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${destDir}' -Force"`, { stdio: 'ignore' });
        } else {
            execSync(`unzip -o "${archivePath}" -d "${destDir}"`, { stdio: 'ignore' });
        }
    } else if (archivePath.endsWith('.tar.gz') || archivePath.endsWith('.tgz')) {
        execSync(`tar -xzf "${archivePath}" -C "${destDir}"`, { stdio: 'ignore' });
    }
}

function httpGetJson(urlStr: string): Promise<any> {
    return new Promise((resolve, reject) => {
        const options = {
            headers: { 'User-Agent': 'vscode-hlsl-shaderlab-extended' }
        };
        https.get(urlStr, options, (res) => {
            if (res.statusCode && res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
                return httpGetJson(res.headers.location).then(resolve, reject);
            }
            if (res.statusCode && (res.statusCode < 200 || res.statusCode >= 300)) {
                return reject(new Error(`HTTP ${res.statusCode}: ${res.statusMessage}`));
            }
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => {
                try {
                    resolve(JSON.parse(data));
                } catch (e) {
                    reject(e);
                }
            });
        }).on('error', reject);
    });
}

function downloadFile(urlStr: string, destPath: string): Promise<void> {
    return new Promise((resolve, reject) => {
        const file = fs.createWriteStream(destPath);
        const options = {
            headers: { 'User-Agent': 'vscode-hlsl-shaderlab-extended' }
        };
        https.get(urlStr, options, (res) => {
            if (res.statusCode && res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
                file.close();
                try { fs.unlinkSync(destPath); } catch {}
                return downloadFile(res.headers.location, destPath).then(resolve, reject);
            }
            if (res.statusCode && (res.statusCode < 200 || res.statusCode >= 300)) {
                file.close();
                try { fs.unlinkSync(destPath); } catch {}
                return reject(new Error(`HTTP ${res.statusCode}: ${res.statusMessage}`));
            }
            res.pipe(file);
            file.on('finish', () => file.close(() => resolve()));
        }).on('error', (err) => {
            file.close();
            try { fs.unlinkSync(destPath); } catch {}
            reject(err);
        });
    });
}
