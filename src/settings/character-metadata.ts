import type { CharacterMetadataDraft } from './types'

export const CHARACTER_NAME_LIMIT = 64
export const CHARACTER_DESCRIPTION_LIMIT = 280

const UNICODE_WHITESPACE = /\p{White_Space}+/gu
const LEADING_OR_TRAILING_UNICODE_WHITESPACE = /^\p{White_Space}+|\p{White_Space}+$/gu
const NAME_CONTROL_CHARACTER = /[\u0000-\u001f\u007f-\u009f]/u
const DESCRIPTION_CONTROL_CHARACTER = /[\u0000-\u0009\u000b-\u001f\u007f-\u009f]/u

export function characterLength(value: string): number {
  return Array.from(value).length
}

export function normalizeCharacterName(value: string): string {
  return value
    .replace(UNICODE_WHITESPACE, ' ')
    .replace(LEADING_OR_TRAILING_UNICODE_WHITESPACE, '')
}

export function normalizeCharacterDescription(value: string): string {
  return value
    .replace(/\r\n?/gu, '\n')
    .replace(LEADING_OR_TRAILING_UNICODE_WHITESPACE, '')
}

export function normalizeCharacterMetadata(draft: CharacterMetadataDraft): CharacterMetadataDraft {
  return {
    displayName: normalizeCharacterName(draft.displayName),
    description: normalizeCharacterDescription(draft.description),
  }
}

export function validateCharacterMetadata(draft: CharacterMetadataDraft): string | undefined {
  if (!draft.displayName) return '请输入角色名称。'
  if (characterLength(draft.displayName) > CHARACTER_NAME_LIMIT) {
    return `角色名称不能超过 ${CHARACTER_NAME_LIMIT} 个字符。`
  }
  if (NAME_CONTROL_CHARACTER.test(draft.displayName)) {
    return '角色名称不能包含控制字符。'
  }
  if (characterLength(draft.description) > CHARACTER_DESCRIPTION_LIMIT) {
    return `角色介绍不能超过 ${CHARACTER_DESCRIPTION_LIMIT} 个字符。`
  }
  if (DESCRIPTION_CONTROL_CHARACTER.test(draft.description)) {
    return '角色介绍包含不支持的控制字符。'
  }
  return undefined
}
