import { mount } from 'svelte'
import App from './svelte/app.svelte'
import './styles/base.css'
import './styles/layout.css'

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
