import { copyFileSync, mkdirSync, readdirSync, existsSync, readFileSync } from 'fs'
import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')
const pkg = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf-8'))
const ver = pkg.version

const releaseDir = resolve(root, 'src-tauri/target/release')
const outDir = resolve(releaseDir, `v${ver}`)
if (!existsSync(outDir)) mkdirSync(outDir, { recursive: true })

const exe = resolve(releaseDir, 'ai-grading-assistant.exe')
if (existsSync(exe)) copyFileSync(exe, resolve(outDir, `AI阅卷助手_v${ver}_x64.exe`))

const bundleDir = resolve(releaseDir, 'bundle')
if (existsSync(bundleDir)) {
  for (const sub of readdirSync(bundleDir)) {
    for (const file of readdirSync(resolve(bundleDir, sub))) {
      copyFileSync(resolve(bundleDir, sub, file), resolve(outDir, file))
    }
  }
}

console.log(`Artifacts copied to ${outDir}`)
