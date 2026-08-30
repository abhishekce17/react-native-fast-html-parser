package com.margelo.nitro.fasthtmlparser
  
import com.facebook.proguard.annotations.DoNotStrip

@DoNotStrip
class FastHtmlParser : HybridFastHtmlParserSpec() {
  override fun multiply(a: Double, b: Double): Double {
    return a * b
  }
}
