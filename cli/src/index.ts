import createExpress from "express"
import {
    resolveSource
} from "./source.js"


const PORT = 5000
const app = createExpress()

app.get('*', async (req, res) => {
    let url = new URL(`${req.protocol}://${req.get('host')}${req.originalUrl}`)
    let source = await resolveSource(url);

    if ('status' in source) {
        // Invalid source
        res.sendStatus(source.status)
    } else {
        // Valid source
        if (source.mime)
            res.setHeader('Content-Type', source.mime)

        res.status(200)
            .send(source.body)
    }
})

app.listen(PORT, () => {
    console.log(`Running at http://localhost:${PORT}`)
})
