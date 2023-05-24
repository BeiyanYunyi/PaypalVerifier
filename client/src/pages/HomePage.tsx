import { loadScript } from '@paypal/paypal-js';
import { onMount, type Component, Show, createSignal } from 'solid-js';

const HomePage: Component = () => {
  const [payed, setPayed] = createSignal(false);
  const afterApprove = async (orderID: string) => {
    const req = new Request('/api/v1/orderComplete', {
      method: 'POST',
      body: JSON.stringify({ order_id: orderID }),
      headers: { 'Content-Type': 'application/json' },
    });
    const res2 = await fetch(req);
    const json = await res2.json();
    console.log(json);
    return json;
  };
  onMount(async () => {
    const paypal = await loadScript({
      'client-id': import.meta.env.CLIENT_ID,
      components: 'buttons,messages',
      'enable-funding': 'card',
      intent: 'authorize',
    });
    paypal!.Buttons!({
      createOrder: (_data, actions) =>
        actions.order.create({
          purchase_units: [
            {
              amount: {
                value: '0.01',
                currency_code: 'USD',
                breakdown: {
                  item_total: { value: '0.01', currency_code: 'USD' },
                },
              },
              items: [
                {
                  name: '验证',
                  description: '验证',
                  quantity: '1',
                  unit_amount: { value: '0.01', currency_code: 'USD' },
                  category: 'DIGITAL_GOODS',
                },
              ],
            },
          ],
          application_context: { landing_page: 'LOGIN' },
        }),
      onApprove: async (_, actions) => {
        const res = await actions.order!.authorize();
        afterApprove(res.id);
        setPayed(true);
      },
    }).render('#container');
  });

  return (
    <div class="flex flex-col items-center">
      <Show when={!payed()}>
        <div id="container" />
      </Show>
    </div>
  );
};

export default HomePage;
