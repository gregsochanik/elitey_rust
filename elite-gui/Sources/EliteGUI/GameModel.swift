import Foundation

enum Feedback {
    case success(String)
    case failure(String)
}

@MainActor
final class GameModel: ObservableObject {
    private let game = Game(commanderName: "Commander")

    @Published private(set) var market: MarketData
    @Published var feedback: Feedback?

    init() {
        market = try! game.market()
    }

    func buy(_ id: String, quantity: UInt32) {
        trade { try game.buy(commodityId: id, quantity: quantity) }
    }

    func sell(_ id: String, quantity: UInt32) {
        trade { try game.sell(commodityId: id, quantity: quantity) }
    }

    private func trade(_ action: () throws -> TradeResult) {
        do {
            feedback = .success(try action().message)
        } catch {
            feedback = .failure(error.localizedDescription)
        }
        if let refreshed = try? game.market() {
            market = refreshed
        }
    }
}
