export function is_valid_slug(value) {
   return value.match(/^[a-z-]+$/)
}

export function parse_int(value) {
    return parseInt(value) || ''
}
