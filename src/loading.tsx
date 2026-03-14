import { createRoot } from 'react-dom/client'
import { Loader } from './components/organisms/loader'

import './styles/index.css'

const rootElement = document.getElementById('app')!
const root = createRoot(rootElement)

root.render(<Loader />)
