# react-native-fast-html-parser

A lightning-fast HTML parser library for React Native that converts raw HTML fragments into a structured JSON AST tree. Built on a high-performance Rust core and integrated via direct C++ JSI HostObjects, it achieves pure zero-copy shared-memory access on-device. This avoids standard string/JSON serialization overhead and Hermes garbage collection pauses during render loops.

## Installation


```sh
npm install react-native-fast-html-parser react-native-nitro-modules

> `react-native-nitro-modules` is required as this library relies on [Nitro Modules](https://nitro.margelo.com/).
```


## Usage


```js
import { multiply } from 'react-native-fast-html-parser';

// ...

const result = multiply(3, 7);
```


## Contributing

- [Development workflow](CONTRIBUTING.md#development-workflow)
- [Sending a pull request](CONTRIBUTING.md#sending-a-pull-request)
- [Code of conduct](CODE_OF_CONDUCT.md)

## License

MIT

---

Made with [create-react-native-library](https://github.com/callstack/react-native-builder-bob)
