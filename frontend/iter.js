export function* entries(obj) {
    for(var key in obj) {
        yield [key, obj[key]]
    }
}

export function first_key(obj) {
    for(var k in obj) {
        return k
    }
}

export function entry_list(obj) {
    return Array.from(entries(obj))
}
