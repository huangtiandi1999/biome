/* should not generate diagnostics */
declare global {
    export interface App extends Services {}
    export namespace Ns {
        export interface Empty {}
    }
}
