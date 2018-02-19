export function all_daemons(source) {
    let daemons = {}
    for(let iname in source.images) {
        let img = source.images[iname]
        for(let name in img.daemons || {}) {
            daemons[name] = img.daemons[name]
        }
    }
    return daemons
}

export function process_info(source, process, version) {
    if(!source || !process || !version)
        return;
    let dep = source.deployments[version]
    if(!dep)
        return;
    for(let cont of dep.containers) {
        let img = source.images[cont];
        let pro = img && img.daemons && img.daemons[process]
        if(pro) {
            return pro
        }
    }
}
