import { loadFile } from "./fs.js"


interface ResolvedSource {
    status: number,
    body: string
}



async function resolveSource(url: URL): Promise<ResolvedSource> {
    const path = url.pathname.slice(1)
    let locator: string

    if (path == '') {
        locator = 'src/pages/index.html'
    } else {
        locator = `src/static/${path}`
    }

    const contents = await loadFile(locator)

    return contents.mapOrElse(
        (e) => {
            console.error(e)
    
            return {
                status: 500,
                body: "Internal server error"
            }
        },
        (res) => ({
            status: 200,
            body: res
        })
    )
}

export { resolveSource }
