import { JSDOM } from 'jsdom'
import { resolveSource } from './source.js'
import { Err, Ok, Result } from 'oxide.ts'
import path from 'path'
import {
    ErrorTypeError
} from "./utils.js"
import e from 'express'


function getElementFromHref(dom: JSDOM, href: string): Element | null {
    if (href.endsWith(".css")) {
        let el = dom.window.document.createElement('style')
        dom.window.document.head.appendChild(el)
        return el
    } else {
        return null
    }
}

export async function processHtml(url: URL, html: string): Promise<Result<string, ProcessHtmlError>> {
    const dom = new JSDOM(html, {
        url: url.toString()
    })

    const document = dom.window.document

    const nib_imports = document.getElementsByTagName('nib-import')

    console.log(`${nib_imports.length} imports found`)

    for (let i = 0; i < nib_imports.length; i++) {
        const nib_item = nib_imports[i]

        const href = nib_item.getAttribute('href')

        if (!href) return Err(ProcessHtmlErrorCode.MISSING_HREF)

        let newEl = getElementFromHref(dom, href)

        if (!newEl) return Err(ProcessHtmlErrorCode.NO_HREF_ELEMENT)

        const elPath = path.resolve(url.pathname, href)

        // Passing be reference messes things up
        let elUrl = new URL(url)
        elUrl.pathname = elPath

        const contents = await resolveSource(elUrl)

        if ('status' in contents) {
            if (contents.status == 404) {
                return Err(ProcessHtmlErrorCode.SOURCE_NOT_FOUND)
            } else {
                return Err(ProcessHtmlErrorCode.SOURCE_INTERNAL)
            }
        }

        newEl.innerHTML = contents.body

        nib_item.remove()
    }

    return Ok(dom.serialize())
}

type ProcessHtmlError =
    ErrorTypeError |
    ProcessHtmlErrorCode

export enum ProcessHtmlErrorCode {
    MISSING_HREF,
    NO_HREF_ELEMENT,
    SOURCE_INTERNAL,
    SOURCE_NOT_FOUND
}
