import { unwrap } from '../common'

const parser = import('../../pkg-thin/index.js').then(p => p.default).then(p => {
  p.init()
  return p
})

export async function validateSet(contractAST: any, data: { [k: string]: any }): Promise<void> {
  return unwrap(JSON.parse((await parser).validate_set(JSON.stringify(contractAST), JSON.stringify(data))))
}
