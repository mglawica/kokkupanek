import {CANCEL} from 'khufu-runtime'
import {set} from './value'


let GLOBAL_COUNTER = 0

export function form() {
    let listeners = []
    let remote = {}
    let store = {
        remote,
        getState() {
            return root.getState()
        },
        dispatch(action) {
            switch(action.type) {
                case 'loading':
                    remote.loading = true
                    remote.errors = null
                    break
                case 'success':
                    remote.loading = false
                    break
                case 'error':
                    remote.loading = false
                    remote.errors = action.errors
                    break
                case CANCEL:
                    listeners = null;  // save some memory
                    break;
            }
            store._trigger()
        },
        subscribe(callback) {
            listeners.push(callback)
            return function() {
                let idx = listeners.indexOf(callback);
                if(idx >= 0) {
                    listeners.splice(idx, 1);
                }
            }
        },
        _trigger() {
            for(let list of listeners) {
                list()
            }
        },
    }
    var root = field(store, null, {})
    store._root = root
    return store
}

export function field(owner, name, default_value) {
    let value = default_value
    let was_input = false
    let fields = new Map()
    let id = 'field_' + (GLOBAL_COUNTER += 1)
    name = name || id
    let form = owner._form
    if(!form) {
        form = owner
    }
    let store = {
        id,
        _form: form,
        _add_field(name, field) {
            fields.set(name, field)
        },
        _remove_field(name, field) {
            if(fields.get(name, field)) {
                fields.delete(name)
            }
        },
        items() {
            return fields.entries()
        },
        getState() {
            if(fields.size > 0) {
                if(Array.isArray(default_value)) {
                    return Array.from(fields.values())
                        .map(x => x.getState())
                        .filter(x => x != null)
                } else {
                    let r = {}
                    for(let [k, v] of fields.entries()) {
                        r[k] = v.getState()
                    }
                    return r
                }
            } else {
                return value
            }
        },
        subscribe(callback) {
            return form.subscribe(callback)
        },
        dispatch(action) {
            switch(action.type) {
                case 'set':
                    value = action.value
                    break;
                case 'input':
                    value = action.value
                    break;
                case 'set_default':
                    if(value == default_value) {
                        value = action.value
                    }
                    default_value = action.value
                    break;
                case 'field_action':
                    let f = fields.get(action.field)
                    f.dispatch(action.action)
                    break;
                case 'add_item':
                    field(store)
                    break;
                case CANCEL:
                    if(owner != form) owner._remove_field(name, store)
                    break;
            }
            form._trigger()
        },
    }
    if(owner == form) {
        owner = form._root
    }
    if(owner) {
        owner._add_field(name, store)
    }
    return store;
}

export function add_item() {
    return {type: 'add_item'}
}

export function field_action(field, action) {
    return {type: 'field_action', field, action}
}

export function input(val) {
    return { type: 'input', value: val }
}

export function set_default(val) {
    return { type: 'set_default', value: val }
}

export function submit(event) {
    event.preventDefault()
    return { type: 'submit' }
}
