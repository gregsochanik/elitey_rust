import SwiftUI

extension MarketRowData: Identifiable {}

struct MarketScreen: View {
    @ObservedObject var model: GameModel
    @State private var selection: MarketRowData.ID?
    @State private var quantity: UInt32 = 1

    private var selected: MarketRowData? {
        model.market.rows.first { $0.id == selection }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            header
            table
            controls
            feedbackLine
        }
        .padding(16)
    }

    private var header: some View {
        HStack {
            Label("\(model.market.systemName) Market", systemImage: "chart.bar.fill")
                .font(.title2.bold())
            Spacer()
            Label("\(model.market.credits) cr", systemImage: "banknote")
                .foregroundStyle(.green)
            Label(
                "\(model.market.cargoUsed)/\(model.market.cargoCapacity) t",
                systemImage: "shippingbox"
            )
        }
    }

    private var table: some View {
        Table(model.market.rows, selection: $selection) {
            TableColumn("Commodity", value: \.name)
            TableColumn("Buy") { Text("\($0.buyPrice) cr").monospacedDigit() }
            TableColumn("Sell") { Text("\($0.sellPrice) cr").monospacedDigit() }
            TableColumn("Available") { Text("\($0.available) t").monospacedDigit() }
            TableColumn("vs Base") { row in
                Text(String(format: "%+d%%", row.percentVsBase))
                    .monospacedDigit()
                    .foregroundStyle(color(for: row.percentVsBase))
            }
            TableColumn("Hold") { row in
                if row.held > 0 {
                    Text(row.paid.map { "\(row.held) t @ \($0) cr" } ?? "\(row.held) t")
                        .foregroundStyle(.cyan)
                }
            }
        }
    }

    private var controls: some View {
        HStack {
            Stepper("Tons: \(quantity)", value: $quantity, in: 1...100)
                .frame(width: 140)
            Button("Buy") { act { model.buy($0.id, quantity: quantity) } }
                .disabled(selected == nil)
            Button("Sell") { act { model.sell($0.id, quantity: quantity) } }
                .disabled((selected?.held ?? 0) == 0)
            Spacer()
            Text(selected.map { "Selected: \($0.name)" } ?? "Select a commodity")
                .foregroundStyle(.secondary)
        }
    }

    @ViewBuilder
    private var feedbackLine: some View {
        switch model.feedback {
        case .success(let message):
            Label(message, systemImage: "checkmark.circle.fill").foregroundStyle(.green)
        case .failure(let message):
            Label(message, systemImage: "xmark.octagon.fill").foregroundStyle(.red)
        case nil:
            Text(" ")
        }
    }

    private func act(_ action: (MarketRowData) -> Void) {
        if let row = selected { action(row) }
    }

    private func color(for percent: Int32) -> Color {
        if percent <= -10 { return .green }
        if percent >= 10 { return .red }
        return .primary
    }
}
