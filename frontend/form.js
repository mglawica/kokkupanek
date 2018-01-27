import {CANCEL} from 'khufu-runtime'


let GLOBAL_COUNTER = 0


export class Form {
    constructor() {
        this._fields = {}
        this._listeners = []
        this._form = this
    }
    field(name, default_value) {
        return new Field(this, name, default_value)
    }
    list_field(name, default_value) {
        return new ListField(this, name, default_value)
    }
    getState() {
        let r = {}
        for(var k in this._fields) {
            r[k] = this._fields[k].getState()
        }
        return r
    }
    dispatch(action) {
        switch(action.type) {
            case CANCEL:
                break;
        }
    }
    subscribe(callback) {
        this._listeners.push(callback)
        return function() {
            let idx = this._listeners.indexOf(callback);
            if(idx >= 0) {
                this._listeners.splice(idx, 1);
            }
        }
    }
    _add_field(name, field) {
        this._fields[name] = field
    }
    _remove_field(name, field) {
        if(this._fields[name] == field) {
            delete this._fields[name]
        }
    }
    _trigger() {
        for(let list of this._listeners) {
            list()
        }
    }
}

export class Field {
    constructor(owner, name, default_value) {
        this._name = name
        this._owner = owner
        this._form = owner._form
        this._value = default_value
        this._subfields = []
        this.id = 'field_' + (GLOBAL_COUNTER += 1)
        owner._add_field(name || this.id, this)
    }
    getState() {
        return this._value
    }
    subscribe(callback) {
        return this._form.subscribe(callback)
    }
    dispatch(action) {
        switch(action.type) {
            case 'set':
                this._value = action.value
                break;
            case 'set_field':
                this._value[action.key] = action.value
                break;
            case CANCEL:
                this._owner._remove_field(this._name, this)
                break;
        }
        this._form._trigger()
    }
}

export class ListField extends Field {
    constructor(owner, name, default_value) {
        super(owner, name, default_value)
        this._fields = new Map()
        default_value.forEach((x, i) => new Field(this, null, x))
    }
    getState() {
        console.log("VALUES", Array.from(this._fields.values()))
        return Array.prototype.map.call(
            this._fields.values(), x => x.getState())
    }
    items() {
        return this._fields.entries()
    }
    _add_field(name, field) {
        this._fields.set(name, field)
    }
    _remove_field(name, field) {
        if(this._fields.get(name) == field) {
            this._fields.delete(name)
        }
    }
}

export function form() {
    return new Form()
}

export function set_field(key, value) {
    return {action: 'set_field', key, value}
}
