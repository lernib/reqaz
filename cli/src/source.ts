import { getPathFromUrl, loadFile, ReadFileErrorCode } from "./fs.js"
import { processHtml, ProcessHtmlErrorCode } from "./html.js"
import { lookup as mimeLookup } from "mime-types"
import * as path from 'path'


interface BasicSource {
    url: URL
}

export interface ValidSource extends BasicSource {
    body: string,
    mime?: string,
}

export interface InvalidSource extends BasicSource {
    status: 404 | 500
}

type Source = ValidSource | InvalidSource

async function resolveSource(url: URL): Promise<Source> {
    let p = await getPathFromUrl(url)

    const mime = mimeLookup(p.ext) || undefined
    const basics: BasicSource = {
        url
    }

    const contents = await loadFile(p)

    if (contents.isErr()) {
        // safe
        const e = contents.unwrapErr()

        let status = 500

        if (e == ReadFileErrorCode.FILE_NOT_FOUND) {
            status = 404
        }
        
        return {
            ...basics,
            status
        } as Source
    } else {
        // safe
        let res = contents.unwrap()
   
        if (p.ext == '.html') {
            const processed = await processHtml(url, res)

            if (processed.isErr()) {
                const e = processed.unwrapErr()

                switch (e) {
                    case ProcessHtmlErrorCode.SOURCE_NOT_FOUND:
                        return {
                            ...basics,
                            status: 404
                        } as InvalidSource as Source
                    case ProcessHtmlErrorCode.MISSING_HREF:
                    case ProcessHtmlErrorCode.NO_HREF_ELEMENT:
                    case ProcessHtmlErrorCode.SOURCE_INTERNAL:
                        return {
                            ...basics,
                            status: 500
                        } as InvalidSource as Source
                }
            } else {
                res = processed.unwrap()
            }
        }

        return {
            ...basics,
            body: res,
            mime
        } as ValidSource as Source
    }
}

export { resolveSource }
