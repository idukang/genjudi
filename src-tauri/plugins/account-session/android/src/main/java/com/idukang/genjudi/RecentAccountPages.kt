package com.idukang.genjudi

import android.app.ActivityManager
import android.content.ComponentCallbacks2
import android.content.Context
import android.content.MutableContextWrapper
import android.content.res.Configuration
import android.graphics.Bitmap
import android.net.Uri
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import android.webkit.RenderProcessGoneDetail
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.webkit.WebViewCompat

internal object RecentAccountPages : ComponentCallbacks2 {
    private const val MAX_ACCOUNT_PAGES = 2
    private const val IDLE_TIMEOUT_MS = 120_000L
    private data class Entry(val view: WebView, val expiresAt: Long)

    private val pages = linkedMapOf<Pair<String, String>, Entry>()
    private val handler = Handler(Looper.getMainLooper())
    private val expire = Runnable { trim(MAX_ACCOUNT_PAGES) }
    private var memoryManager: ActivityManager? = null

    private val parkedClient = object : WebViewClient() {
        override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest) =
            request.isForMainFrame

        override fun onPageStarted(view: WebView, url: String?, favicon: Bitmap?) = discardView(view)

        override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean {
            discardView(view)
            return true
        }
    }

    private fun canRetain(context: Context): Boolean {
        val app = context.applicationContext
        val manager = memoryManager ?: app.getSystemService(ActivityManager::class.java).also {
            memoryManager = it
            app.registerComponentCallbacks(this)
        }
        return !manager.isLowRamDevice && !ActivityManager.MemoryInfo().also(manager::getMemoryInfo).lowMemory
    }

    private fun isReusable(view: WebView, spec: AndroidAccountSessionSpec): Boolean {
        val uri = view.url?.let(Uri::parse) ?: return false
        val entryHost = Uri.parse(spec.entryUrl).host ?: return false
        return AccountSessionCoordinator.hostMatchesAllowed(uri, listOf(entryHost)) &&
            AccountSessionCoordinator.hostMatchesAllowed(uri, spec.allowedHosts) &&
            runCatching { WebViewCompat.getProfile(view).name == spec.uuid }.getOrDefault(false)
    }

    fun take(context: Context, spec: AndroidAccountSessionSpec): WebView? {
        if (!canRetain(context)) {
            trim(0)
            return null
        }
        trim(MAX_ACCOUNT_PAGES)
        val view = pages.remove(spec.providerId to spec.uuid)?.view
        trim(MAX_ACCOUNT_PAGES - 1)
        if (view == null) return null
        if (spec.isColdStart || !isReusable(view, spec)) {
            view.destroy()
            return null
        }
        return view
    }

    fun park(view: WebView, spec: AndroidAccountSessionSpec) {
        if (!canRetain(view.context)) {
            trim(0)
            view.destroy()
            return
        }
        if (!isReusable(view, spec)) {
            view.destroy()
            return
        }
        view.webViewClient = parkedClient
        view.onPause()
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            view.setRendererPriorityPolicy(WebView.RENDERER_PRIORITY_WAIVED, true)
        }
        val context = view.context as MutableContextWrapper
        context.baseContext = context.applicationContext
        val key = spec.providerId to spec.uuid
        pages.remove(key)?.view?.destroy()
        pages[key] = Entry(view, SystemClock.elapsedRealtime() + IDLE_TIMEOUT_MS)
        trim(MAX_ACCOUNT_PAGES)
    }

    fun discardProfile(uuid: String) {
        pages.keys.filter { it.second == uuid }.forEach { pages.remove(it)?.view?.destroy() }
    }

    private fun discardView(view: WebView) {
        val key = pages.entries.firstOrNull { it.value.view === view }?.key ?: return
        pages.remove(key)
        view.destroy()
    }

    private fun trim(limit: Int) {
        handler.removeCallbacks(expire)
        while (pages.isNotEmpty()) {
            val first = pages.entries.first()
            val remaining = first.value.expiresAt - SystemClock.elapsedRealtime()
            if (pages.size <= limit && remaining > 0) {
                handler.postDelayed(expire, remaining)
                return
            }
            pages.remove(first.key)
            first.value.view.destroy()
        }
    }

    override fun onTrimMemory(level: Int) {
        if (level >= ComponentCallbacks2.TRIM_MEMORY_RUNNING_LOW) trim(0)
    }

    override fun onLowMemory() = trim(0)
    override fun onConfigurationChanged(newConfig: Configuration) = trim(0)
}
