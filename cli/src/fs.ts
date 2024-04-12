import * as fs from "fs/promises"
import * as path from "path"
import { Result, Ok, Err } from "oxide.ts"
import {
    errorTypeErrorEncapsulate,
    ErrorTypeError
} from "./utils.js"


const CWD = process.cwd()

async function pathExists(path: string): Promise<boolean> {
    try {
        await fs.stat(path)
        return true
    } catch {
        return false
    }
}

export async function pathIsDirectory(path: string): Promise<boolean> {
    if (await pathExists(path)) {
        let stat = await fs.stat(path)

        return stat.isDirectory()
    }

    return false
}

export async function loadFile(locator: string): Promise<Result<string, ReadFileError>> {
    const p = path.join(CWD, locator)

    if (!await pathExists(p)) {
        return Err(ReadFileErrorCode.FILE_NOT_FOUND)
    }

    return (await Result.safe(fs.readFile(p, 'utf-8'))).mapErr(errorTypeErrorEncapsulate)
}

export async function loadSrcFile(locator: string): Promise<Result<string, ReadFileError>> {
    return await loadFile(path.join('src', locator))
}

type ReadFileError = ErrorTypeError | ReadFileErrorCode
export enum ReadFileErrorCode {
    FILE_NOT_FOUND
}
