import { Route, Routes } from '@solidjs/router';
import { type Component } from 'solid-js';
// eslint-disable-next-line import/no-extraneous-dependencies
import '@unocss/reset/tailwind-compat.css';
import 'virtual:uno.css';
import './App.css';
import HomePage from './pages/HomePage';
import Plum from './components/Plum';

const App: Component = () => (
  <>
    <header class="mb-2 p-2">
      <h1 class="text-2xl select-none">验证</h1>
    </header>
    <Plum />
    <main>
      <Routes>
        <Route path="/" component={HomePage} />{' '}
      </Routes>
    </main>
  </>
);

export default App;
