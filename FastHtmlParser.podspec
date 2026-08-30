require "json"

package = JSON.parse(File.read(File.join(__dir__, "package.json")))

Pod::Spec.new do |s|
  s.name         = "FastHtmlParser"
  s.version      = package["version"]
  s.summary      = package["description"]
  s.homepage     = package["homepage"]
  s.license      = package["license"]
  s.authors      = package["author"]

  s.platforms    = { :ios => min_ios_version_supported }
  s.source       = { :git => "https://github.com/abhishekce17/react-native-fast-html-parser.git", :tag => "#{s.version}" }

  s.source_files = [
    "ios/**/*.{swift}",
    "ios/**/*.{m,mm}",
    "cpp/**/*.{hpp,cpp}",
  ]

  # Target XCConfig to resolve and link the precompiled Rust static library
  s.pod_target_xcconfig = {
    'LIBRARY_SEARCH_PATHS[sdk=iphonesimulator*]' => '$(PODS_TARGET_SRCROOT)/ios/libs/sim',
    'LIBRARY_SEARCH_PATHS[sdk=iphoneos*]' => '$(PODS_TARGET_SRCROOT)/ios/libs/device',
    'OTHER_LDFLAGS' => '-lhtml_2_json'
  }

  s.dependency 'React-jsi'
  s.dependency 'React-callinvoker'

  load 'nitrogen/generated/ios/FastHtmlParser+autolinking.rb'
  add_nitrogen_files(s)

  install_modules_dependencies(s)
end
