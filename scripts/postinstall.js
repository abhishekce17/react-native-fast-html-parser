#!/usr/bin/env node
/**
 * Postinstall script for react-native-fast-html-parser
 *
 * Downloads precompiled Rust native binaries from GitHub Releases
 * so developers do not need Rust installed on their machines.
 */

'use strict';

const https = require('https');
const fs = require('fs');
const path = require('path');

const pkg = require('../package.json');
const VERSION = pkg.version;
const REPO = 'abhishekce17/react-native-fast-html-parser';
const BASE_URL =
  'https://github.com/' + REPO + '/releases/download/v' + VERSION;
const PKG_ROOT = path.resolve(__dirname, '..');

const BINARIES = [
  {
    name: 'libhtml_2_json-ios-device.a',
    dest: path.join(PKG_ROOT, 'ios', 'libs', 'device', 'libhtml_2_json.a'),
  },
  {
    name: 'libhtml_2_json-ios-sim.a',
    dest: path.join(PKG_ROOT, 'ios', 'libs', 'sim', 'libhtml_2_json.a'),
  },
  {
    name: 'libhtml_2_json-android-arm64-v8a.so',
    dest: path.join(
      PKG_ROOT,
      'android',
      'src',
      'main',
      'jniLibs',
      'arm64-v8a',
      'libhtml_2_json.so'
    ),
  },
  {
    name: 'libhtml_2_json-android-armeabi-v7a.so',
    dest: path.join(
      PKG_ROOT,
      'android',
      'src',
      'main',
      'jniLibs',
      'armeabi-v7a',
      'libhtml_2_json.so'
    ),
  },
  {
    name: 'libhtml_2_json-android-x86.so',
    dest: path.join(
      PKG_ROOT,
      'android',
      'src',
      'main',
      'jniLibs',
      'x86',
      'libhtml_2_json.so'
    ),
  },
  {
    name: 'libhtml_2_json-android-x86_64.so',
    dest: path.join(
      PKG_ROOT,
      'android',
      'src',
      'main',
      'jniLibs',
      'x86_64',
      'libhtml_2_json.so'
    ),
  },
];

function download(url, destPath) {
  return new Promise(function (resolve, reject) {
    fs.mkdirSync(path.dirname(destPath), { recursive: true });
    const file = fs.createWriteStream(destPath);

    function get(currentUrl) {
      https
        .get(
          currentUrl,
          {
            headers: {
              'User-Agent': 'react-native-fast-html-parser-postinstall',
            },
          },
          function (res) {
            if (res.statusCode === 301 || res.statusCode === 302) {
              file.close();
              return get(res.headers.location);
            }
            if (res.statusCode !== 200) {
              file.close();
              try {
                fs.unlinkSync(destPath);
              } catch {}
              return reject(
                new Error(
                  'Failed to download ' +
                    currentUrl +
                    ': HTTP ' +
                    res.statusCode
                )
              );
            }
            res.pipe(file);
            file.on('finish', function () {
              file.close(function () {
                const stat = fs.statSync(destPath);
                if (stat.size === 0) {
                  try {
                    fs.unlinkSync(destPath);
                  } catch {}
                  return reject(
                    new Error(
                      'Downloaded file is empty: ' + path.basename(destPath)
                    )
                  );
                }
                resolve();
              });
            });
          }
        )
        .on('error', function (err) {
          file.close();
          try {
            fs.unlinkSync(destPath);
          } catch {}
          reject(err);
        });
    }

    get(url);
  });
}

async function main() {
  console.log(
    '\n[react-native-fast-html-parser] Downloading native binaries v' +
      VERSION +
      '...'
  );
  console.log('[react-native-fast-html-parser] Source: ' + BASE_URL + '\n');

  let failed = false;

  for (const binary of BINARIES) {
    if (fs.existsSync(binary.dest)) {
      console.log('  \u2713 Already exists: ' + binary.name);
      continue;
    }

    const url = BASE_URL + '/' + binary.name;
    process.stdout.write('  \u2193 Downloading ' + binary.name + '...');

    try {
      await download(url, binary.dest);
      console.log(' \u2713');
    } catch (err) {
      console.log(' \u2717');
      console.error('    Error: ' + err.message);
      failed = true;
    }
  }

  // Assemble iOS XCFramework on macOS machines
  if (process.platform === 'darwin') {
    const xcframeworkPath = path.join(
      PKG_ROOT,
      'ios',
      'libs',
      'libhtml_2_json.xcframework'
    );
    if (!fs.existsSync(xcframeworkPath)) {
      console.log('  ⚙ Assembling iOS XCFramework...');
      const devicePath = path.join(
        PKG_ROOT,
        'ios',
        'libs',
        'device',
        'libhtml_2_json.a'
      );
      const simPath = path.join(
        PKG_ROOT,
        'ios',
        'libs',
        'sim',
        'libhtml_2_json.a'
      );

      try {
        const { execSync } = require('child_process');
        execSync(
          'xcodebuild -create-xcframework -library "' +
            devicePath +
            '" -library "' +
            simPath +
            '" -output "' +
            xcframeworkPath +
            '"',
          { stdio: 'ignore' }
        );
        console.log('  ✓ XCFramework assembled successfully.');
      } catch (err) {
        console.error('  ✗ Failed to assemble XCFramework: ' + err.message);
        process.exit(1);
      }
    } else {
      console.log('  ✓ XCFramework already exists.');
    }
  }

  if (failed) {
    console.error(
      [
        '',
        '[react-native-fast-html-parser] Some binaries failed to download.',
        '  This may happen if:',
        '    - You are offline',
        '    - Release v' + VERSION + ' has not been published yet on GitHub',
        '',
        '  Manually download them from:',
        '    https://github.com/' + REPO + '/releases/tag/v' + VERSION,
        '',
        '  Place them in:',
        '    ios/libs/device/libhtml_2_json.a',
        '    ios/libs/sim/libhtml_2_json.a',
        '    android/src/main/jniLibs/<ABI>/libhtml_2_json.so',
        '',
      ].join('\n')
    );
    process.exit(1);
  }

  console.log('\n[react-native-fast-html-parser] All binaries ready.\n');
}

main();
