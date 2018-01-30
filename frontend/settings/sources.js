import {set} from '../value'
import {add_item, field_action, set_default} from '../form'

export function extract_comment(key) {
    let comment = key.split(' ', 3)[2];
    if(comment) {
        return field_action('comment', set_default(comment))
    } else {
        return field_action('comment', set_default(''))
    }
}

export function check() {
    return {type: 'check'}
}

function _check(store) {
    let cur = store.getState();
    let last = cur[cur.length-1]
    if(!last || last.key) {
        store.dispatch(add_item())
    }
}

export function keep_empty_line(store) {
    _check(store)
    return next => action => {
        switch(action.type) {
            case 'check':
                _check(store)
                break;
        }
        return next(action)
    }
}
