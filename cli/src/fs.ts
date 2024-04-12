import * as fs from "fs/promises"
import * as pathStd from "path"
import { Result, Err } from "oxide.ts"
import {
    errorTypeErrorEncapsulate,
    ErrorTypeError
} from "./utils.js"


// Recreating the path export because the lack of features
// in the standard library is an endless source of
// secondhand embarassment
let path = {
    ...pathStd,
    unparse: unparsePath,
    exists: pathExists
}

const CWD = process.cwd()


/**
 * Unparse a path. Why is this not in the standard library? We
 * will never ever know.
 * 
 * @param p The parsed path
 * @returns A string that is compatible with NodeStd
 */
function unparsePath(p: pathStd.ParsedPath): string {
    return `${p.dir}${path.sep}${p.base}`
}

/**
 * Safely get when a path was last modified, or an error if it doesn't exist.
 * 
 * @param p The path to test
 * @returns The Date last modified or an error
 */
export async function getPathLastModified(p: pathStd.ParsedPath): Promise<Result<Date, StatFileError>> {
    async function unsafeOperation() {
        const stat = await fs.stat(path.unparse(p))
        return stat.mtime
    }

    return (await Result.safe(unsafeOperation())).mapErr(errorTypeErrorEncapsulate)
}

/**
 * Converts a URL to a path on the filesystem
 * 
 * @param url The URL passed to the request
 * @returns The absolute path of the file associated with the URL
 */
export async function getPathFromUrl(url: URL): Promise<pathStd.ParsedPath> {
    // Remove starting slash
    // GET /            :. pathname = ''
    // GET /home        :. pathname = 'home'
    // GET /img/a.jpg   :. pathname = 'img/a.jpg'
    const pathname = url.pathname.slice(1)

    // Check if page exists at this path
    const pages_path = path.parse(path.join(CWD, 'src/pages', pathname, 'index.html'));
    if (await pathExists(pages_path)) {
        return pages_path
    }

    // Return a path based at src/static
    return path.parse(path.join(CWD, 'src/static', pathname))
}

/**
 * Check if a path exists, because TypeScript doesn't have this
 * built in for some god forsaken reason.
 * 
 * @param p The path to the file 
 * @returns A boolean indicating existence
 */
async function pathExists(p: pathStd.ParsedPath): Promise<boolean> {
    try {
        await fs.stat(path.unparse(p))
        return true
    } catch {
        return false
    }
}

/**
 * Check if a path is a directory.
 * Whatd'ya know, not in the standard library anywhere...
 * 
 * @param p The parsed path to check
 * @returns A boolean indicating whether this is a directory or not
 */
export async function pathIsDirectory(p: pathStd.ParsedPath): Promise<boolean> {
    if (await pathExists(p)) {
        let stat = await fs.stat(path.unparse(p))

        return stat.isDirectory()
    }

    return false
}

/**
 * Read a file from the filesystem. If a URL is passed, it is converted
 * into a path first.
 * 
 * @param locator The locator used to find the path
 * @returns The file contents, or an error
 */
export async function loadFile(locator: URL | pathStd.ParsedPath): Promise<Result<string, ReadFileError>> {
    const p = locator instanceof URL ?
        await getPathFromUrl(locator) :
        locator

    if (!await pathExists(p)) {
        return Err(ReadFileErrorCode.FILE_NOT_FOUND)
    }

    return (await Result.safe(fs.readFile(path.unparse(p), 'utf-8'))).mapErr(errorTypeErrorEncapsulate)
}

// TODO: Create unified FileError
type ReadFileError = ErrorTypeError | ReadFileErrorCode
export enum ReadFileErrorCode {
    FILE_NOT_FOUND
}

type StatFileError = ErrorTypeError | StatFileErrorCode
export enum StatFileErrorCode {
    FILE_NOT_FOUND
}
