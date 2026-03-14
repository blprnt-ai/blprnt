import { flow, makeAutoObservable } from 'mobx'
import { PersonalityModel } from '@/lib/models/personality.model'

export class PersonalitiesViewModel {
  personalities: Map<string, PersonalityViewModel> = new Map()
  isLoading = false
  loadError: string | null = null

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
    void this.reload()
  }

  get default() {
    return this.asArray.find((p) => p.isDefault)
  }

  get asArray() {
    return Array.from(this.personalities.values()).toSorted((a, b) => (a.name < b.name ? -1 : 1))
  }

  get systemPersonalities() {
    return this.asArray.filter((personality) => !personality.isUserDefined)
  }

  get userPersonalities() {
    return this.asArray.filter((personality) => personality.isUserDefined)
  }

  reload = flow(function* (this: PersonalitiesViewModel) {
    this.isLoading = true
    this.loadError = null

    try {
      const result: PersonalityModel[] = yield PersonalityModel.list()
      this.personalities = new Map(
        result.map((personality) => [personality.id, new PersonalityViewModel(this, personality)]),
      )
    } catch (error) {
      this.loadError = error instanceof Error ? error.message : 'Failed to load personalities.'
    } finally {
      this.isLoading = false
    }
  })

  create = flow(function* (this: PersonalitiesViewModel, args: ReturnType<PersonalityViewModel['toCreateArgs']>) {
    const result = yield PersonalityModel.create(args.name, args.description, args.systemPrompt)
    const viewmodel = new PersonalityViewModel(this, result)
    this.personalities.set(result.id, viewmodel)
    return viewmodel
  })

  delete = flow(function* (this: PersonalitiesViewModel, id: string) {
    const personality = this.personalities.get(id)
    if (!personality) return
    yield personality.model.delete()
    this.personalities.delete(id)
  })
}

export class PersonalityViewModel {
  isDirty = false
  showPreview = true

  constructor(
    public readonly personalities: PersonalitiesViewModel,
    public readonly model: PersonalityModel,
  ) {
    makeAutoObservable(this, { model: false, personalities: false }, { autoBind: true })
  }

  get id() {
    return this.model.id
  }

  get name() {
    return this.model.name
  }

  get isDefault() {
    return this.model.isDefault
  }

  setName = (name: string) => {
    this.model.name = name
  }

  get description() {
    return this.model.description
  }

  setDescription = (description: string) => {
    this.model.description = description
  }

  get systemPrompt() {
    return this.model.systemPrompt
  }

  setSystemPrompt = (systemPrompt: string) => {
    this.model.systemPrompt = systemPrompt
  }

  get isUserDefined() {
    return this.model.isUserDefined
  }

  get createdAt() {
    return this.model.createdAt
  }

  get updatedAt() {
    return this.model.updatedAt
  }

  get isValid() {
    return this.name.trim().length >= 5 && this.description.trim().length >= 5 && this.systemPrompt.trim().length >= 5
  }

  setIsDirty = (isDirty: boolean) => {
    this.isDirty = isDirty
  }

  setShowPreview = (showPreview: boolean) => {
    this.showPreview = showPreview
  }

  create = flow(function* (this: PersonalityViewModel) {
    const personality: PersonalityViewModel = yield this.personalities.create(this.toCreateArgs())
    return personality
  })

  update = flow(function* (this: PersonalityViewModel) {
    yield this.model.update({
      description: this.description,
      name: this.name,
      systemPrompt: this.systemPrompt,
    })
  })

  delete = flow(function* (this: PersonalityViewModel) {
    yield this.personalities.delete(this.id)
  })

  toCreateArgs = () => {
    return {
      description: this.description,
      name: this.name,
      systemPrompt: this.systemPrompt,
    }
  }
}
