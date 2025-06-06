/* should not generate diagnostics */
class User {
    protected field: string;
    get name() {
        return 'John Doe';
    }
    set name(value) {}
    constructor() {}
    get #age() {
        return 30;
    }
    set #age(age) {}
    method() {}
    // Ignores getters without setters
    get isHuman() {
        return true;
    }
    // Ignores setters without getters
    set enabled(enabled: boolean) {}
}

const user = {
    field: 'field',
    get name() {
        return 'John Doe';
    },
    set name(value) {},
    method() {},
    get age() {
        return 30;
    },
    set age(age) {},
    get isHuman() {
        return true;
    },
    set enabled(enabled: boolean) {}
};

type UserType = {
    field: string;
    get name(): string;
    set name(value: string);
    method(): void;
    get age(): number;
    set age(age: number);
    get isHuman(): boolean;
    set enabled(enabled: boolean);
}

interface UserInterface {
    field: string;
    get name(): string;
    set name(value: string);
    method(): void;
    get age(): number;
    set age(age: number);
    get isHuman(): boolean;
    set enabled(enabled: boolean);
}

declare module 'module' {
    export class User {
        protected field: string;
        get name(): string;
        set name(value: string);
        constructor();
        get #age(): number;
        set #age(age: number);
        method(): void;
        get isHuman(): boolean;
        set enabled(enabled: boolean);
    }
}
