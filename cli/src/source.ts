import { loadFile, ReadFileErrorCode } from "./fs.js"
import { processHtml, ProcessHtmlErrorCode } from "./html.js"
import { lookup as mimeLookup } from "mime-types"
import * as path from 'path'
import chalk from "chalk"


export interface ValidSource {
    body: string,
    mime?: string,
}

export interface InvalidSource {
    status: 404 | 500
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

    if (contents.isErr()) {
        // safe
        const e = contents.unwrapErr()

        let status = 500

        if (e == ReadFileErrorCode.FILE_NOT_FOUND) {
            status = 404
        }

        logSourceRequest(status, url.pathname)
        
        return {
            status
        } as Source
    } else {
        // safe
        let res = contents.unwrap()
   
        if (locator.endsWith('.html')) {
            const processed = await processHtml(url, res)

            if (processed.isErr()) {
                const e = processed.unwrapErr()

                switch (e) {
                    case ProcessHtmlErrorCode.SOURCE_NOT_FOUND:
                        return {
                            status: 404
                        }
                    case ProcessHtmlErrorCode.MISSING_HREF:
                    case ProcessHtmlErrorCode.NO_HREF_ELEMENT:
                    case ProcessHtmlErrorCode.SOURCE_INTERNAL:
                        return {
                            status: 500
                        }
                }
            } else {
                res = processed.unwrap()
            }
        }

        logSourceRequest(200, url.pathname)

        return {
            body: res,
            mime
        } as Source
    }
}

export { resolveSource }
