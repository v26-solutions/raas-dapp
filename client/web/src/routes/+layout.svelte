<script>
  import Wallet from '../components/Wallet.svelte'

</script>

<nav>
  <a href="/">RaaS Dashboard</a>

  <Wallet />
</nav>

<slot></slot>

<style>

nav {
  background-color: #000;
  padding-left: 30px;
  padding-right: 10px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

a {
  color: white;
  text-align: center;
  padding: 14px 16px;
  text-decoration: none;
}

a:hover {
  background-color: #333;
}

</style>