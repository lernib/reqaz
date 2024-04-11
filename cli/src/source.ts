import { loadFile } from "./fs.js"
import { lookup as mimeLookup } from "mime-types"
import * as path from 'path'

interface ResolvedSource {
    status: number,
    body: string,
    mime?: string
}



async function resolveSource(url: URL): Promise<ResolvedSource> {
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
            console.error(e)
    
            return {
                status: 500,
                body: "Internal server error",
                mime
            }
        },
        (res) => ({
            status: 200,
            body: res,
            mime
        })
    )
}

export { resolveSource }
