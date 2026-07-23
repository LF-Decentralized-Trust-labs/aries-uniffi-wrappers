import XCTest
@testable import IndyVdr

final class BasicTests: XCTestCase {
    let printLength = 200
    func testFeatures() async throws {
        guard let genesisUrl = Bundle.module.url(forResource: "genesis_sov_buildernet", withExtension: "txn") else {
            XCTFail("Genesis file not found")
            return
        }

        let pool = try openPool(transactionsPath: genesisUrl.path, transactions: nil, nodeWeights: nil)
        print("Status:", try await pool.getStatus())

        try await pool.refresh()
        print("Status after refresh:", try await pool.getStatus())

        let ledger = Ledger()

        let testReq = """
        {
            "operation": {"data": 1, "ledgerId": 1, "type": "3"},
            "protocolVersion": 2,
            "reqId": 123,
            "identifier": "LibindyDid111111111111"
        }
        """
        var req = try ledger.buildCustomRequest(body: testReq)
        print("Custom request body:", try req.body())

        let sigIn = try req.signatureInput()
        print("Custom request signature input:", sigIn)

        req = try ledger.buildGetTxnAuthorAgreementRequest(submitterDid: nil, data: nil)
        print("buildGetTxnAuthorAgreementRequest:", try await pool.submitRequest(request: req).prefix(printLength))

        req = try ledger.buildGetAcceptanceMechanismsRequest(submitterDid: nil, timestamp: nil, version: nil)
        print("buildGetAcceptanceMechanismsRequest:", try await pool.submitRequest(request: req).prefix(printLength))

        let acceptance = try ledger.prepareTxnAuthorAgreementAcceptance(
            text: "acceptance text",
            version: "1.1.1",
            taaDigest: nil,
            mechanism: "manual",
            time: UInt64(Date().timeIntervalSince1970))
        req = try ledger.buildGetTxnRequest(submitterDid: nil, ledgerType: .domain, seqNo: 15)
        try req.setTxnAuthorAgreementAcceptance(acceptance: acceptance)
        try req.setEndorser(endorser: "V4SGRU86Z58d6TV7PBUe6f")
        try req.setMultiSignature(identifier: "V4SGRU86Z58d6TV7PBUe6f", signature: Data("sig".utf8))
        print("Request with TAA acceptance and endorser:", try req.body())

        req = try ledger.buildGetTxnRequest(submitterDid: nil, ledgerType: .domain, seqNo: 1)
        print("buildGetTxnRequest:", try await pool.submitRequest(request: req).prefix(printLength))

        req = try ledger.buildGetSchemaRequest(submitterDid: nil, schemaId: "6qnvgJtqwK44D8LFYnV5Yf:2:relationship.dflow:1.0.0")
        print("Get schema request:", try req.body())

        req = try ledger.buildGetCredDefRequest(submitterDid: nil, credDefId: "A9Rsuu7FNquw8Ne2Smu5Nr:3:CL:15:tag")
        print("Get cred def request:", try req.body())

        let revocId = "L5wx9FUxCDpFJEdFc23jcn:4:L5wx9FUxCDpFJEdFc23jcn:3:CL:1954:default:CL_ACCUM:c024e30d-f3eb-42c5-908a-ff885667588d"

        req = try ledger.buildGetRevocRegDefRequest(submitterDid: nil, revRegId: revocId)
        print("Get revoc reg def request:", try req.body())

        req = try ledger.buildGetRevocRegRequest(submitterDid: nil, revRegId: revocId, timestamp: 1)
        print("Get revoc reg request:", try req.body())

        req = try ledger.buildGetRevocRegDeltaRequest(submitterDid: nil, revRegId: revocId, from: nil, to: 1)
        print("Get revoc reg delta request:", try req.body())
        
        try await pool.closePool()
    }
}
