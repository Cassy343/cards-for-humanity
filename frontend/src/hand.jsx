class Hand extends React.Component {
    constructor(props) {
        super(props);

        this.state = {
            cards: [],
        }

        global_var.update_hand = (hand) => {
            this.setState({cards: hand})
        }
    }

    render() {
        const cards = this.state.cards;
        let output;
        for(let i = 0; i < cards.length; i++) {
            let card = cards[i];
            output += <Card text={card.text} index={i}/>
        }

        return <div>{output}</div>
    }
}