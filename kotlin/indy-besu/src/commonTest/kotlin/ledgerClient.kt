import indy_besu_vdr.LedgerClient
import indy_besu_vdr.ContractConfig
import indy_besu_vdr.Status
import kotlin.test.Test
import kotlin.test.assertTrue
import kotlinx.coroutines.runBlocking

class LedgerClientTest {
    @Test
    fun testPing() = runBlocking {
        val client = LedgerClient(
            chainId = 1234uL,
            nodeAddress = "http://localhost:8888",
            contractConfigs = listOf<ContractConfig>(),
            network = null,
            quorumConfig = null
        )
        val pingStatus = client.ping()
        assertTrue(pingStatus.status is Status.Err)
    }
}
