export type Root = Node[]

export type Node = Contract | { kind: string }

export interface Contract {
  kind: 'contract'
  namespace: Namespace
  name: string
  attributes: ContractAttribute[]
}

export interface Namespace {
  kind: 'namespace',
  value: string
}

export type ContractAttribute = Property | Index | Method | Directive

export interface Property {
  kind: 'property',
  name: string,
  type: Type
  directives: Directive[]
}

export interface Index {
  kind: 'index',
  fields: IndexField[]
}

export interface IndexField {
  direction: IndexFieldDirection
  fieldPath: string[]
}

export type IndexFieldDirection = 'asc' | 'desc'

export interface Method {
  kind: 'method'
  name: string
  code: string
  attributes: MethodAttribute[]
}

export type MethodAttribute = Parameter | ReturnValue | Directive

export interface Parameter {
  kind: 'parameter'
  name: string
  type: Type
  required: boolean
  directives: Directive[]
}

export type Type = Primitive | Object | Array | Map | ForeignRecord | PublicKey

export interface Primitive {
  kind: 'primitive',
  value: 'string' | 'number' | 'boolean' | 'bytes'
}

export interface Array {
  kind: 'array',
  value: Primitive[]
}

export interface Map {
  kind: 'map',
  key: Primitive
  value: Primitive | ForeignRecord
}

export interface Object {
  kind: 'object',
  fields: ObjectField[]
}

export interface ObjectField {
  name: string
  type: Type
  required: boolean
}

export interface ForeignRecord {
  kind: 'foreignrecord',
  contract: string
}

export interface PublicKey {
  kind: 'publickey',
}

export interface Directive {
  kind: 'directive'
  name: string
  arguments: DirectiveArgument[]
}

export type DirectiveArgument = FieldReference

export interface FieldReference {
  kind: 'fieldreference',
  path: string[]
}

export interface ReturnValue {
  kind: 'returnvalue'
  name: string
  type: Type
}
