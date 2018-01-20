export function repr(x) {
    return JSON.stringify(x)
}

export function pretty(x) {
    return JSON.stringify(x, null, 2)
}

export function log(...args) {
    console.log(...args)
}

