import type { AppState } from './types'
import { DEFAULT_PET_ID, DEFAULT_SETTINGS } from './types'

export const MOCK_STATE: AppState = {
  settings: { ...DEFAULT_SETTINGS },
  selectedPetId: DEFAULT_PET_ID,
  pets: [
    {
      id: DEFAULT_PET_ID,
      displayName: '晴檐',
      description: '戴草帽的温柔桌面伙伴。',
      renderType: 'sprite-atlas-v2',
      source: 'builtin',
      deletable: false,
      assetPath: '',
      thumbnailPath: '',
      dialogues: { idle: ['今天也慢慢来。'], waving: ['我在这里。'], jumping: ['被你发现啦。'] },
    },
    {
      id: 'qpet-z2-violet-whisper',
      displayName: '堇语',
      description: '藏着一枝小花的安静伙伴。',
      renderType: 'sprite-atlas-v2',
      source: 'builtin',
      deletable: false,
      assetPath: '',
      thumbnailPath: '',
      dialogues: { idle: ['花香很轻。'], waving: ['要一起休息一下吗？'] },
    },
    {
      id: 'qpet-gu-candied-haw',
      displayName: '糖葫芦',
      description: '带着糖葫芦的橙金汉服伙伴。',
      renderType: 'static-image-v1',
      source: 'builtin',
      deletable: false,
      assetPath: '',
      thumbnailPath: '',
      dialogues: { idle: ['要尝一颗吗？'], jumping: ['甜甜的！'] },
    },
  ],
}
