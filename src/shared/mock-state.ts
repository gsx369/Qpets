import type { AppState } from './types'
import { DEFAULT_PET_ID, DEFAULT_SETTINGS } from './types'

export const MOCK_STATE: AppState = {
  settings: { ...DEFAULT_SETTINGS },
  selectedPetId: DEFAULT_PET_ID,
  pets: [
    {
      id: DEFAULT_PET_ID,
      displayName: '赵大美女',
      description: '大美女，大设计师，P图大师，复合型人才',
      metadataCustomized: false,
      renderType: 'sprite-atlas-v2',
      source: 'builtin',
      deletable: false,
      assetPath: '',
      thumbnailPath: '',
      dialogues: { idle: ['今天也慢慢来。'], waving: ['我在这里。'], jumping: ['被你发现啦。'] },
    },
    {
      id: 'qpet-z2-violet-whisper',
      displayName: '赵二美女',
      description: '二美女，二设计师，P图二师，复合型人才',
      metadataCustomized: false,
      renderType: 'sprite-atlas-v2',
      source: 'builtin',
      deletable: false,
      assetPath: '',
      thumbnailPath: '',
      dialogues: { idle: ['花香很轻。'], waving: ['要一起休息一下吗？'] },
    },
    {
      id: 'qpet-gu-candied-haw',
      displayName: '俺滴老妹',
      description: '世界上最可爱的人',
      metadataCustomized: false,
      renderType: 'sprite-atlas-v2',
      source: 'builtin',
      deletable: false,
      assetPath: '',
      thumbnailPath: '',
      dialogues: { idle: ['要尝一颗吗？'], jumping: ['甜甜的！'] },
    },
  ],
}
