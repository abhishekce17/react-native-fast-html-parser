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
    "ios/**/*.{m,mm}",
    "cpp/**/*.{hpp,cpp}",
  ]

  s.vendored_frameworks = 'ios/libs/libhtml_2_json.xcframework'

  s.dependency 'React-jsi'
  s.dependency 'React-callinvoker'

  load 'nitrogen/generated/ios/FastHtmlParser+autolinking.rb'
  add_nitrogen_files(s)

  install_modules_dependencies(s)

  # Force C++17 standard to prevent Fabric compiler standard mismatches
  s.attributes_hash["pod_target_xcconfig"] ||= {}
  s.attributes_hash["pod_target_xcconfig"]["CLANG_CXX_LANGUAGE_STANDARD"] = "c++17"
  s.attributes_hash["user_target_xcconfig"] ||= {}
  s.attributes_hash["user_target_xcconfig"]["CLANG_CXX_LANGUAGE_STANDARD"] = "c++17"
end
