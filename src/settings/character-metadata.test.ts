import { describe, expect, it } from 'vitest'

import {
  characterLength,
  normalizeCharacterMetadata,
  validateCharacterMetadata,
} from './character-metadata'

describe('character metadata', () => {
  it('normalizes name whitespace and description newlines', () => {
    expect(normalizeCharacterMetadata({
      displayName: '  晴   檐\n',
      description: '  第一行\r\n第二行  ',
    })).toEqual({
      displayName: '晴 檐',
      description: '第一行\n第二行',
    })
  })

  it('counts unicode characters instead of UTF-16 code units', () => {
    expect(characterLength('晴🌸')).toBe(2)
    expect(validateCharacterMetadata({ displayName: '🌸'.repeat(64), description: '' })).toBeUndefined()
  })

  it('rejects empty and overlong metadata', () => {
    expect(validateCharacterMetadata({ displayName: '', description: '' })).toContain('请输入')
    expect(validateCharacterMetadata({ displayName: '名'.repeat(65), description: '' })).toContain('64')
    expect(validateCharacterMetadata({ displayName: '晴檐', description: '介'.repeat(281) })).toContain('280')
  })

  it('matches the backend whitespace and control-character rules', () => {
    expect(normalizeCharacterMetadata({
      displayName: '\u0085晴\t檐\u0085',
      description: '\u0085第一行\r第二行\u0085',
    })).toEqual({
      displayName: '晴 檐',
      description: '第一行\n第二行',
    })
    expect(normalizeCharacterMetadata({
      displayName: '\ufeff晴檐\ufeff',
      description: '\ufeff介绍\ufeff',
    })).toEqual({
      displayName: '\ufeff晴檐\ufeff',
      description: '\ufeff介绍\ufeff',
    })
    expect(validateCharacterMetadata({ displayName: '晴\0檐', description: '' })).toContain('控制字符')
    expect(validateCharacterMetadata({ displayName: '晴檐', description: '第一行\t第二行' })).toContain('控制字符')
  })
})
