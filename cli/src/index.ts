import createExpress from "express"
import etag from "etag"
import {
    resolveSource
} from "./source.js"
import { logSourceRequest } from "./utils.js"

// typescript was a mistake

const PORT = 5000
const app = createExpress()

// Disable default etagging, we can handle it
app.disable('etag')



app.get('*', async (req, res) => {
    let url = new URL(`${req.protocol}://${req.get('host')}${req.originalUrl}`)
    let source = await resolveSource(url);

    if ('status' in source) {
        // Invalid source
        logSourceRequest(source.status, source.url)
        res.sendStatus(source.status)
    } else {
        // Valid source
        if (source.mime)
            res.setHeader('Content-Type', source.mime)

        const sourceEtag = etag(source.body)
        res.setHeader('ETag', sourceEtag)
        if (req.headers["if-none-match"] == sourceEtag) {
            logSourceRequest(304, source.url)

            res.sendStatus(304)
        } else {
            logSourceRequest(200, source.url)

            res.status(200)
                .send(source.body)
        }
    }
})

app.listen(PORT, () => {
    console.log(`Running at http://localhost:${PORT}`)
})
