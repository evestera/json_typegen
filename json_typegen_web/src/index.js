import { polyfill } from './polyfill';

polyfill().then(() => import("../../json_typegen_wasm/pkg")).then(module => {
  const render = () => {
    const typename = document.getElementById('typename').value;
    let input = document.getElementById('input').value;
    if (input === '') {
      input = '{}';
    }
    const options = ({
      "output_mode": document.getElementById('outputmode').value,
    });
    const result = module.run(typename, input, JSON.stringify(options));
    document.getElementById('target').innerHTML = result
        .replace(/&/g,'&amp;')
        .replace(/</g,'&lt;')
        .replace(/>/g,'&gt;');
  };

  document.getElementById('typename').onkeyup = render;
  document.getElementById('input').onkeyup = render;
  document.getElementById('outputmode').onchange = render;

  render();
});
