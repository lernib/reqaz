import * as fs from "fs/promises"
import * as path from "path"
import { Result, Ok, Err } from "oxide.ts"


const CWD = process.cwd()

async function pathExists(path: string): Promise<boolean> {
    try {
        await fs.stat(path)
        return true
    } catch {
        return false
    }
}

async function pathIsDirectory(path: string): Promise<boolean> {
    if (await pathExists(path)) {
        let stat = await fs.stat(path)

        return stat.isDirectory()
    }

    return false
}

async function loadFile(locator: string): Promise<Result<string, Error>> {
    return await Result.safe(fs.readFile(path.join(CWD, locator), 'utf-8'))
}

async function loadSrcFile(locator: string): Promise<Result<string, Error>> {
    return await loadFile(path.join('src', locator))
}

export {
    pathExists, pathIsDirectory,
    loadFile, loadSrcFile
}
