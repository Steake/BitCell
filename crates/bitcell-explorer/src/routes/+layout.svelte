<script>
import '../app.css';
</script>

<div class="app">
<header>
<nav>
<h1><a href="/">🔍 BitCell Block Explorer</a></h1>
<div class="nav-links">
<a href="/">Blocks</a>
<a href="/search">Search</a>
</div>
</nav>
</header>

<main>
<slot />
</main>

<footer>
<p>BitCell Block Explorer - Connected to live node</p>
</footer>
</div>
