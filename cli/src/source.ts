import { loadFile, ReadFileErrorCode } from "./fs.js"
import { lookup as mimeLookup } from "mime-types"
import * as path from 'path'
import chalk from "chalk"


export interface ValidSource {
    body: string,
    mime?: string,
}

export interface InvalidSource {
    status: number
}

type Source = ValidSource | InvalidSource

function colorStatus(status: number): string {
    if (status < 200) {
        return chalk.blue.bold(status)
    } else if (status < 300) {
        return chalk.green.bold(status)
    } else if (status < 400) {
        return chalk.yellow.bold(status)
    } else if (status < 500) {
        return chalk.red.bold(status)
    } else {
        return chalk.magentaBright.bold(status)
    }
}

function logSourceRequest(status: number, locator: string) {
    const statusStr = colorStatus(status)

    console.info(`[${statusStr}] ${locator}`)
}

async function resolveSource(url: URL): Promise<Source> {
    const p = url.pathname.slice(1)
    let locator: string

    if (p == '') {
        locator = 'src/pages/index.html'
    } else {
        locator = `src/static/${p}`
    }

    const mime = mimeLookup(path.extname(locator)) || undefined

    const contents = await loadFile(locator)

    return contents.mapOrElse(
        (e) => {
            let status = 500

            if (e == ReadFileErrorCode.FILE_NOT_FOUND) {
                status = 404
            }

            logSourceRequest(status, url.pathname)
            
            return {
                status
            } as Source
        },
        (res) => {
            logSourceRequest(200, url.pathname)

            return {
                body: res,
                mime
            } as Source
        }
    )
}

export { resolveSource }
