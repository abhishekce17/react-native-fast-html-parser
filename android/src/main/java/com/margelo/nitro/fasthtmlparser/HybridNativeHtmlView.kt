package com.margelo.nitro.fasthtmlparser

import android.content.Context
import android.graphics.Color
import android.graphics.Typeface
import android.text.method.LinkMovementMethod
import android.util.TypedValue
import android.view.View
import android.widget.TextView
import androidx.core.view.AccessibilityDelegateCompat
import androidx.core.view.ViewCompat
import androidx.core.view.accessibility.AccessibilityNodeInfoCompat
import androidx.core.widget.TextViewCompat
import com.margelo.nitro.NitroModules

class HybridNativeHtmlView(
    private val context: Context = NitroModules.applicationContext
        ?: throw IllegalStateException("NitroModules.applicationContext is null")
) : HybridNativeHtmlViewSpec() {
    private val textView: TextView by lazy {
        TextView(context).apply {
            setTextIsSelectable(true)
            movementMethod = LinkMovementMethod.getInstance()
            setBackgroundColor(Color.TRANSPARENT)
            ViewCompat.setAccessibilityDelegate(this, object : AccessibilityDelegateCompat() {
                override fun onInitializeAccessibilityNodeInfo(host: View, info: AccessibilityNodeInfoCompat) {
                    super.onInitializeAccessibilityNodeInfo(host, info)
                    info.className = TextView::class.java.name
                }
            })
        }
    }

    override val view: View
        get() = textView

    private var isUpdateScheduled = false
    private val mainHandler = android.os.Handler(android.os.Looper.getMainLooper())

    override var html: String? = null
        set(value) {
            field = value
            setNeedsContentUpdate()
        }

    override var baseStyle: NativeTextStyle? = null
        set(value) {
            field = value
            setNeedsContentUpdate()
        }

    override var tagsStyles: Map<String, NativeTextStyle>? = null
        set(value) {
            field = value
            setNeedsContentUpdate()
        }

    override var selectable: Boolean? = true
        set(value) {
            field = value
            textView.setTextIsSelectable(value ?: true)
        }

    override var themeMode: String? = null
        set(value) {
            field = value
            setNeedsContentUpdate()
        }

    override var onLinkPress: ((url: String) -> Unit)? = null
        set(value) {
            field = value
            setNeedsContentUpdate()
        }

    override var onContentSizeChange: ((height: Double) -> Unit)? = null

    private fun setNeedsContentUpdate() {
        if (isUpdateScheduled) return
        isUpdateScheduled = true
        mainHandler.post {
            isUpdateScheduled = false
            updateContent()
        }
    }

    override fun getTextContent(): String {
        return textView.text?.toString() ?: ""
    }

    private fun updateContent() {
        val rawHtml = html ?: ""
        
        // 1. Configure base text appearance on TextView
        val resolvedColor = SpannableHtmlEngine.parseColor(baseStyle?.color) ?: when (themeMode) {
            "dark" -> Color.WHITE
            "light" -> Color.parseColor("#1E293B")
            else -> Color.parseColor("#1E293B")
        }
        textView.setTextColor(resolvedColor)

        baseStyle?.fontSize?.let { size ->
            if (size > 0) {
                textView.setTextSize(TypedValue.COMPLEX_UNIT_DIP, size.toFloat())
            }
        }

        baseStyle?.lineHeight?.let { lh ->
            if (lh > 0) {
                val density = context.resources.displayMetrics.density
                val lhPx = (lh * density).toInt()
                TextViewCompat.setLineHeight(textView, lhPx)
            }
        }

        baseStyle?.fontFamily?.let { fam ->
            if (fam.isNotBlank()) {
                try {
                    textView.typeface = Typeface.create(fam, Typeface.NORMAL)
                } catch (_: Exception) {}
            }
        }

        // 2. Apply OpenType Font Features if provided
        baseStyle?.fontFeatureSettings?.let { features ->
            if (features.isNotBlank()) {
                textView.paint.fontFeatureSettings = features
            }
        }

        // 3. Build styled spannable with custom spans and tagsStyles
        val spannable = SpannableHtmlEngine.buildSpannable(context, rawHtml, baseStyle, tagsStyles, themeMode, onLinkPress)
        textView.text = spannable

        // 4. Measure layout height and report back to React Native Yoga
        textView.post {
            val density = context.resources.displayMetrics.density
            val hPx = textView.layout?.height ?: textView.measuredHeight
            if (hPx > 0 && density > 0) {
                val hDp = hPx.toDouble() / density.toDouble()
                onContentSizeChange?.invoke(hDp)
            }
        }
    }
}
