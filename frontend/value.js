export function init(val) {
    return { type: 'init', value: val }
}
export function value(state=undefined, action) {
    switch(action.type) {
        case 'init':
            if(state === undefined) {
                return action.value;
            } else {
                return state;
            }
        case 'set':
            return action.value;
        default: return state;
    }
}

export function set(val) {
    return { type: 'set', value: val }
}
