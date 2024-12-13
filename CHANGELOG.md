# Changelog

## [0.4.0](https://github.com/kaans/mqtli/compare/v0.3.0...v0.4.0) (2024-12-13)


### Features

* add option to output raw bytes as lossy utf8 ([5d4da69](https://github.com/kaans/mqtli/commit/5d4da69f39e7a14cb8fce9d25c3cba3d3fc2580f))
* change all payload formats to native types and convert between them ([e287a76](https://github.com/kaans/mqtli/commit/e287a76fee60f8bb4c433288204038a0ce4a4c27))
* **config:** make the presence of the config file optional and use defaults ([497b9bb](https://github.com/kaans/mqtli/commit/497b9bb1b3608c0efedbb22e472355362a4d4269))
* **config:** remove short flags for help and version, rename host flag to -h and add mqtt version flag -v ([e5bcf81](https://github.com/kaans/mqtli/commit/e5bcf8195d9f27df85df1d6c4b05ea0deab00fcb))
* **console:** use color to output to console and for logging ([a4f5f03](https://github.com/kaans/mqtli/commit/a4f5f0388866cfdbc0cea953b03b4d228b461121))
* **mqtt:** create mqtt v311 client ([16a7052](https://github.com/kaans/mqtli/commit/16a70528d9196c21bc14cf675d3a468ab698de0b))
* **mqtt:** enable websocket feature for rumqttc ([955a577](https://github.com/kaans/mqtli/commit/955a577129ca414cddbac933ec53bd65359ca05f))
* **mqtt:** support websockets, including tls ([#28](https://github.com/kaans/mqtli/issues/28)) ([955a577](https://github.com/kaans/mqtli/commit/955a577129ca414cddbac933ec53bd65359ca05f))
* **payload:** Add conversion from yaml to protobuf ([e095301](https://github.com/kaans/mqtli/commit/e095301558bfe0049ea57d3f397eb7893fb54931))
* **payload:** convert from json and yaml to protobuf and vice-versa ([e095301](https://github.com/kaans/mqtli/commit/e095301558bfe0049ea57d3f397eb7893fb54931))
* print config on debug log ([49c0aea](https://github.com/kaans/mqtli/commit/49c0aea23518f990b407b89dcacb997dfd0501e9))
* print protobuf as text ([23591b2](https://github.com/kaans/mqtli/commit/23591b2587c1482753fb7ce0faf4317635d2e275))
* **trigger:** publish trigger based messages with times relative to the start of the program instead of cron based scheduling ([380ade4](https://github.com/kaans/mqtli/commit/380ade4d51cc132d5ed512729627c11910f956e6))
* use raw bytes in base64 ([8d0ca5e](https://github.com/kaans/mqtli/commit/8d0ca5ea3616ad0ec6aae01253c2d78742bb85da))
* use raw bytes in hex ([1de7a0a](https://github.com/kaans/mqtli/commit/1de7a0aea906d56fb013ae011512f00d59fde1d4))
* use raw bytes in text ([91b11fe](https://github.com/kaans/mqtli/commit/91b11fe452ab22e8db8c8def7a37281efb82a6a8))


### Bug Fixes

* examples imports ([deb7ddc](https://github.com/kaans/mqtli/commit/deb7ddc40b2de552a475afc5686af8c9985eed25))
* parse json and yaml from text,base64,hex ([d9f4257](https://github.com/kaans/mqtli/commit/d9f425758b07306aabd466d9761d8fa93d09b752))
* parse yaml and json directly from other types without content attribute ([0b34f9c](https://github.com/kaans/mqtli/commit/0b34f9cec4478f27e10cc6d72d679d70e90c9e0f))
* tests in comments ([2a2b765](https://github.com/kaans/mqtli/commit/2a2b765026fdeed2d4090278c00d551e184ef6bb))
* **trigger:** respect initial delay when publishing messages ([380ade4](https://github.com/kaans/mqtli/commit/380ade4d51cc132d5ed512729627c11910f956e6))
* **trigger:** use count value to limit number of published messages ([380ade4](https://github.com/kaans/mqtli/commit/380ade4d51cc132d5ed512729627c11910f956e6))


### Miscellaneous Chores

* release 0.4.0 ([c279f69](https://github.com/kaans/mqtli/commit/c279f69e3ca9fe35719e00ffc99485a6edfd2f9d))

## [0.3.0](https://github.com/kaans/mqtli/compare/v0.2.0...v0.3.0) (2024-01-09)


### Bug Fixes

* typo in readme ([69b1a2a](https://github.com/kaans/mqtli/commit/69b1a2a89f1fa7a6a194e850be97a7cc3c1aa1dc))


### Miscellaneous Chores

* release 0.3.0 ([3bc6bfa](https://github.com/kaans/mqtli/commit/3bc6bfaf72094b387e82f54514341259c557557b))
