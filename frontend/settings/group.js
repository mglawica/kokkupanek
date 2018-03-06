export function make_slug(cfg, daemons) {
    return (cfg
        .replace(/^.*\//, '')
        .replace(/^lithos\./, '')
        .replace(/\.yaml$/, '')
        .replace(/[^a-z0-9]+/g, ''))
}
