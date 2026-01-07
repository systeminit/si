<template>
  <div>
    <template v-if="loadBillingDetailsReqStatus.isPending">
      <Icon name="loader" size="xl" />
    </template>
    <template v-else-if="loadBillingDetailsReqStatus.isError">
      <ErrorMessage :requestStatus="loadBillingDetailsReqStatus" />
    </template>
    <template v-else-if="draftBillingDetail">
      <div class="flex flex-col gap-sm p-lg">
        <div class="text-2xl font-bold pb-sm">
          Update Your Billing Information
        </div>
        <form>
          <Stack>
            <ErrorMessage :requestStatus="updateBillingDetailsReqStatus" />
            <div class="flex flex-col gap-sm px-sm">
              <Tiles columns="2" spacing="sm" columnsMobile="1">
                <VormInput
                  v-model="draftBillingDetail.firstName"
                  label="First Name"
                  autocomplete="given-name"
                  placeholder="Your first name"
                  required
                  :maxLength="MAX_LENGTH_STANDARD"
                  :regex="NAME_REGEX"
                  regexMessage="First name contains invalid characters or URLs"
                />
                <VormInput
                  v-model="draftBillingDetail.lastName"
                  label="Last Name"
                  autocomplete="last-name"
                  placeholder="Your last name"
                  required
                  :maxLength="MAX_LENGTH_STANDARD"
                  :regex="NAME_REGEX"
                  regexMessage="Last name contains invalid characters or URLs"
                />
              </Tiles>
              <VormInput
                v-model="draftBillingDetail.email"
                label="Email"
                type="email"
                autocomplete="email"
                required
                disabled
                placeholder="ex: yourname@somewhere.com"
              />
            </div>
            <div class="text-xl font-bold py-xs">Company Information</div>
            <div class="flex flex-col gap-sm px-sm">
              <VormInput
                v-model="draftBillingDetail.companyInformation.legalName"
                label="Company Legal Name"
                autocomplete="legal-name"
                placeholder="Type a company legal name"
                :regex="ALLOWED_INPUT_REGEX"
              />
              <VormInput
                v-model="draftBillingDetail.companyInformation.legalNumber"
                label="Company Legal Number"
                autocomplete="legal-number"
                placeholder="Type a company legal number"
                :regex="ALLOWED_INPUT_REGEX"
              />
              <VormInput
                v-model="
                  draftBillingDetail.companyInformation.taxIdentificationNumber
                "
                label="Tax Identification Number"
                autocomplete="tax-identification-number"
                placeholder="Type a tax identificatioon number"
                :regex="ALLOWED_INPUT_REGEX"
              />
              <VormInput
                v-model="draftBillingDetail.companyInformation.phoneNumber"
                label="Phone Number"
                autocomplete="phone-number"
                placeholder="Type a phone number"
                :regex="ALLOWED_INPUT_REGEX"
              />
            </div>
            <div class="text-xl font-bold py-xs">Billing Information</div>
            <div class="flex flex-col gap-sm px-sm">
              <VormInput
                v-model="draftBillingDetail.billingInformation.addressLine1"
                label="Address Line 1"
                required
                autocomplete="address-line-1"
                placeholder="Address Line 1"
                :regex="ALLOWED_INPUT_REGEX"
              />
              <VormInput
                v-model="draftBillingDetail.billingInformation.addressLine2"
                label="Address Line 2"
                autocomplete="address-line-2"
                placeholder="Address Line 2"
                :regex="ALLOWED_INPUT_REGEX"
              />
              <VormInput
                v-model="draftBillingDetail.billingInformation.zipCode"
                label="Zip code"
                required
                autocomplete="zip-code"
                placeholder="Zip Code / Postcode"
                :regex="ALLOWED_INPUT_REGEX"
              />
              <VormInput
                v-model="draftBillingDetail.billingInformation.city"
                label="City"
                required
                autocomplete="city"
                placeholder="City"
                :regex="ALLOWED_INPUT_REGEX"
              />
              <VormInput
                v-model="draftBillingDetail.billingInformation.state"
                label="State"
                autocomplete="state"
                placeholder="State"
                :regex="ALLOWED_INPUT_REGEX"
              />
              <VormInput
                v-model="draftBillingDetail.billingInformation.country"
                :options="countryOptions"
                label="Country"
                required
                autocomplete="country"
                placeholder="Country"
                type="dropdown"
              />
            </div>
          </Stack>
          <div class="flex flex-row pt-md gap-sm items-center flex-wrap">
            <VButton
              tone="neutral"
              variant="solid"
              iconRight="external-link"
              @click="
                openPaymentHandler(draftBillingDetail.customerCheckoutUrl)
              "
              >Add / Update Payment Details</VButton
            >
            <VButton
              tone="neutral"
              variant="solid"
              iconRight="external-link"
              @click="openInvoices(draftBillingDetail.customerPortalUrl)"
              >View Your Invoices</VButton
            >
            <VButton
              class="flex-grow"
              tone="action"
              variant="solid"
              iconRight="chevron--right"
              @click="saveHandler()"
              >Save Billing Details</VButton
            >
          </div>
        </form>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
/* eslint-disable @typescript-eslint/no-non-null-assertion */

import * as _ from "lodash-es";
import { computed, ref, watch, onMounted } from "vue";
import {
  ErrorMessage,
  Icon,
  Tiles,
  Stack,
  useValidatedInputGroup,
  VormInput,
  VButton,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { useHead } from "@vueuse/head";
import { useAuthStore, BillingDetails } from "@/store/auth.store";
import {
  ALLOWED_INPUT_REGEX,
  NAME_REGEX,
  MAX_LENGTH_STANDARD,
} from "@/lib/validations";

const { validationMethods } = useValidatedInputGroup();
const authStore = useAuthStore();
const router = useRouter();

const loadBillingDetailsReqStatus = authStore.getRequestStatus(
  "LOAD_BILLING_DETAILS",
);
const updateBillingDetailsReqStatus = authStore.getRequestStatus(
  "UPDATE_BILLING_DETAILS",
);

useHead({ title: "Billing" });

const billingUser = computed(() => authStore.billingDetail);
const user = computed(() => authStore.user);
const draftBillingDetail = ref<BillingDetails>();

function resetBillingUser() {
  draftBillingDetail.value = _.cloneDeep(billingUser.value!);
}
watch(billingUser, resetBillingUser, { immediate: true });
watch(
  user,
  async () => {
    if (user.value) await authStore.LOAD_BILLING_DETAILS();
  },
  { immediate: true },
);

const openPaymentHandler = async (url: string) => {
  if (!url) return;

  window.location.href = url;
};

const openInvoices = async (url: string) => {
  if (!url) return;

  window.open(url, "_blank");
};

const saveHandler = async () => {
  if (validationMethods.hasError()) return;
  const updateBillingReq = await authStore.UPDATE_BILLING_DETAILS(
    draftBillingDetail.value!,
  );
  if (updateBillingReq.result.success) {
    // We should log that we have updated the billing details in Posthog
    await authStore.LOAD_BILLING_DETAILS();
  }
};

interface CountryOption {
  value: string;
  label: string;
}

const countryOptions: CountryOption[] = [
  { value: "AD", label: "Andorra" },
  { value: "AE", label: "United Arab Emirates" },
  { value: "AF", label: "Afghanistan" },
  { value: "AG", label: "Antigua and Barbuda" },
  { value: "AI", label: "Anguilla" },
  { value: "AL", label: "Albania" },
  { value: "AM", label: "Armenia" },
  { value: "AO", label: "Angola" },
  { value: "AQ", label: "Antarctica" },
  { value: "AR", label: "Argentina" },
  { value: "AS", label: "American Samoa" },
  { value: "AT", label: "Austria" },
  { value: "AU", label: "Australia" },
  { value: "AW", label: "Aruba" },
  { value: "AX", label: "Åland Islands" },
  { value: "AZ", label: "Azerbaijan" },
  { value: "BA", label: "Bosnia and Herzegovina" },
  { value: "BB", label: "Barbados" },
  { value: "BD", label: "Bangladesh" },
  { value: "BE", label: "Belgium" },
  { value: "BF", label: "Burkina Faso" },
  { value: "BG", label: "Bulgaria" },
  { value: "BH", label: "Bahrain" },
  { value: "BI", label: "Burundi" },
  { value: "BJ", label: "Benin" },
  { value: "BL", label: "Saint Barthélemy" },
  { value: "BM", label: "Bermuda" },
  { value: "BN", label: "Brunei Darussalam" },
  { value: "BO", label: "Bolivia" },
  { value: "BQ", label: "Bonaire, Sint Eustatius and Saba" },
  { value: "BR", label: "Brazil" },
  { value: "BS", label: "Bahamas" },
  { value: "BT", label: "Bhutan" },
  { value: "BV", label: "Bouvet Island" },
  { value: "BW", label: "Botswana" },
  { value: "BY", label: "Belarus" },
  { value: "BZ", label: "Belize" },
  { value: "CA", label: "Canada" },
  { value: "CC", label: "Cocos (Keeling) Islands" },
  { value: "CD", label: "Congo, Democratic Republic of the" },
  { value: "CF", label: "Central African Republic" },
  { value: "CG", label: "Congo" },
  { value: "CH", label: "Switzerland" },
  { value: "CI", label: "Côte d'Ivoire" },
  { value: "CK", label: "Cook Islands" },
  { value: "CL", label: "Chile" },
  { value: "CM", label: "Cameroon" },
  { value: "CN", label: "China" },
  { value: "CO", label: "Colombia" },
  { value: "CR", label: "Costa Rica" },
  { value: "CU", label: "Cuba" },
  { value: "CV", label: "Cape Verde" },
  { value: "CW", label: "Curaçao" },
  { value: "CX", label: "Christmas Island" },
  { value: "CY", label: "Cyprus" },
  { value: "CZ", label: "Czech Republic" },
  { value: "DE", label: "Germany" },
  { value: "DJ", label: "Djibouti" },
  { value: "DK", label: "Denmark" },
  { value: "DM", label: "Dominica" },
  { value: "DO", label: "Dominican Republic" },
  { value: "DZ", label: "Algeria" },
  { value: "EC", label: "Ecuador" },
  { value: "EE", label: "Estonia" },
  { value: "EG", label: "Egypt" },
  { value: "EH", label: "Western Sahara" },
  { value: "ER", label: "Eritrea" },
  { value: "ES", label: "Spain" },
  { value: "ET", label: "Ethiopia" },
  { value: "FI", label: "Finland" },
  { value: "FJ", label: "Fiji" },
  { value: "FK", label: "Falkland Islands (Malvinas)" },
  { value: "FM", label: "Micronesia, Federated States of" },
  { value: "FO", label: "Faroe Islands" },
  { value: "FR", label: "France" },
  { value: "GA", label: "Gabon" },
  { value: "GB", label: "United Kingdom" },
  { value: "GD", label: "Grenada" },
  { value: "GE", label: "Georgia" },
  { value: "GF", label: "French Guiana" },
  { value: "GG", label: "Guernsey" },
  { value: "GH", label: "Ghana" },
  { value: "GI", label: "Gibraltar" },
  { value: "GL", label: "Greenland" },
  { value: "GM", label: "Gambia" },
  { value: "GN", label: "Guinea" },
  { value: "GP", label: "Guadeloupe" },
  { value: "GQ", label: "Equatorial Guinea" },
  { value: "GR", label: "Greece" },
  { value: "GS", label: "South Georgia and the South Sandwich Islands" },
  { value: "GT", label: "Guatemala" },
  { value: "GU", label: "Guam" },
  { value: "GW", label: "Guinea-Bissau" },
  { value: "GY", label: "Guyana" },
  { value: "HK", label: "Hong Kong" },
  { value: "HM", label: "Heard Island and McDonald Islands" },
  { value: "HN", label: "Honduras" },
  { value: "HR", label: "Croatia" },
  { value: "HT", label: "Haiti" },
  { value: "HU", label: "Hungary" },
  { value: "ID", label: "Indonesia" },
  { value: "IE", label: "Ireland" },
  { value: "IL", label: "Israel" },
  { value: "IM", label: "Isle of Man" },
  { value: "IN", label: "India" },
  { value: "IO", label: "British Indian Ocean Territory" },
  { value: "IQ", label: "Iraq" },
  { value: "IR", label: "Iran, Islamic Republic of" },
  { value: "IS", label: "Iceland" },
  { value: "IT", label: "Italy" },
  { value: "JE", label: "Jersey" },
  { value: "JM", label: "Jamaica" },
  { value: "JO", label: "Jordan" },
  { value: "JP", label: "Japan" },
  { value: "KE", label: "Kenya" },
  { value: "KG", label: "Kyrgyzstan" },
  { value: "KH", label: "Cambodia" },
  { value: "KI", label: "Kiribati" },
  { value: "KM", label: "Comoros" },
  { value: "KN", label: "Saint Kitts and Nevis" },
  { value: "KP", label: "Korea, Democratic People's Republic of" },
  { value: "KR", label: "Korea, Republic of" },
  { value: "KW", label: "Kuwait" },
  { value: "KY", label: "Cayman Islands" },
  { value: "KZ", label: "Kazakhstan" },
  { value: "LA", label: "Lao People's Democratic Republic" },
  { value: "LB", label: "Lebanon" },
  { value: "LC", label: "Saint Lucia" },
  { value: "LI", label: "Liechtenstein" },
  { value: "LK", label: "Sri Lanka" },
  { value: "LR", label: "Liberia" },
  { value: "LS", label: "Lesotho" },
  { value: "LT", label: "Lithuania" },
  { value: "LU", label: "Luxembourg" },
  { value: "LV", label: "Latvia" },
  { value: "LY", label: "Libya" },
  { value: "MA", label: "Morocco" },
  { value: "MC", label: "Monaco" },
  { value: "MD", label: "Moldova, Republic of" },
  { value: "ME", label: "Montenegro" },
  { value: "MF", label: "Saint Martin (French part)" },
  { value: "MG", label: "Madagascar" },
  { value: "MH", label: "Marshall Islands" },
  { value: "MK", label: "North Macedonia" },
  { value: "ML", label: "Mali" },
  { value: "MM", label: "Myanmar" },
  { value: "MN", label: "Mongolia" },
  { value: "MO", label: "Macao" },
  { value: "MP", label: "Northern Mariana Islands" },
  { value: "MQ", label: "Martinique" },
  { value: "MR", label: "Mauritania" },
  { value: "MS", label: "Montserrat" },
  { value: "MT", label: "Malta" },
  { value: "MU", label: "Mauritius" },
  { value: "MV", label: "Maldives" },
  { value: "MW", label: "Malawi" },
  { value: "MX", label: "Mexico" },
  { value: "MY", label: "Malaysia" },
  { value: "MZ", label: "Mozambique" },
  { value: "NA", label: "Namibia" },
  { value: "NC", label: "New Caledonia" },
  { value: "NE", label: "Niger" },
  { value: "NF", label: "Norfolk Island" },
  { value: "NG", label: "Nigeria" },
  { value: "NI", label: "Nicaragua" },
  { value: "NL", label: "Netherlands" },
  { value: "NO", label: "Norway" },
  { value: "NP", label: "Nepal" },
  { value: "NR", label: "Nauru" },
  { value: "NU", label: "Niue" },
  { value: "NZ", label: "New Zealand" },
  { value: "OM", label: "Oman" },
  { value: "PA", label: "Panama" },
  { value: "PE", label: "Peru" },
  { value: "PF", label: "French Polynesia" },
  { value: "PG", label: "Papua New Guinea" },
  { value: "PH", label: "Philippines" },
  { value: "PK", label: "Pakistan" },
  { value: "PL", label: "Poland" },
  { value: "PM", label: "Saint Pierre and Miquelon" },
  { value: "PN", label: "Pitcairn" },
  { value: "PR", label: "Puerto Rico" },
  { value: "PS", label: "Palestine, State of" },
  { value: "PT", label: "Portugal" },
  { value: "PW", label: "Palau" },
  { value: "PY", label: "Paraguay" },
  { value: "QA", label: "Qatar" },
  { value: "RE", label: "Réunion" },
  { value: "RO", label: "Romania" },
  { value: "RS", label: "Serbia" },
  { value: "RU", label: "Russian Federation" },
  { value: "RW", label: "Rwanda" },
  { value: "SA", label: "Saudi Arabia" },
  { value: "SB", label: "Solomon Islands" },
  { value: "SC", label: "Seychelles" },
  { value: "SD", label: "Sudan" },
  { value: "SE", label: "Sweden" },
  { value: "SG", label: "Singapore" },
  { value: "SH", label: "Saint Helena, Ascension and Tristan da Cunha" },
  { value: "SI", label: "Slovenia" },
  { value: "SJ", label: "Svalbard and Jan Mayen" },
  { value: "SK", label: "Slovakia" },
  { value: "SL", label: "Sierra Leone" },
  { value: "SM", label: "San Marino" },
  { value: "SN", label: "Senegal" },
  { value: "SO", label: "Somalia" },
  { value: "SR", label: "Suriname" },
  { value: "SS", label: "South Sudan" },
  { value: "ST", label: "Sao Tome and Principe" },
  { value: "SV", label: "El Salvador" },
  { value: "SX", label: "Sint Maarten (Dutch part)" },
  { value: "SY", label: "Syrian Arab Republic" },
  { value: "SZ", label: "Eswatini" },
  { value: "TC", label: "Turks and Caicos Islands" },
  { value: "TD", label: "Chad" },
  { value: "TF", label: "French Southern Territories" },
  { value: "TG", label: "Togo" },
  { value: "TH", label: "Thailand" },
  { value: "TJ", label: "Tajikistan" },
  { value: "TK", label: "Tokelau" },
  { value: "TL", label: "Timor-Leste" },
  { value: "TM", label: "Turkmenistan" },
  { value: "TN", label: "Tunisia" },
  { value: "TO", label: "Tonga" },
  { value: "TR", label: "Turkey" },
  { value: "TT", label: "Trinidad and Tobago" },
  { value: "TV", label: "Tuvalu" },
  { value: "TW", label: "Taiwan, Province of China" },
  { value: "TZ", label: "Tanzania, United Republic of" },
  { value: "UA", label: "Ukraine" },
  { value: "UG", label: "Uganda" },
  { value: "UM", label: "United States Minor Outlying Islands" },
  { value: "US", label: "United States" },
  { value: "UY", label: "Uruguay" },
  { value: "UZ", label: "Uzbekistan" },
  { value: "VA", label: "Holy See (Vatican City State)" },
  { value: "VC", label: "Saint Vincent and the Grenadines" },
  { value: "VE", label: "Venezuela, Bolivarian Republic of" },
  { value: "VG", label: "Virgin Islands, British" },
  { value: "VI", label: "Virgin Islands, U.S." },
  { value: "VN", label: "Viet Nam" },
  { value: "VU", label: "Vanuatu" },
  { value: "WF", label: "Wallis and Futuna" },
  { value: "WS", label: "Samoa" },
  { value: "YE", label: "Yemen" },
  { value: "YT", label: "Mayotte" },
  { value: "ZA", label: "South Africa" },
  { value: "ZM", label: "Zambia" },
  { value: "ZW", label: "Zimbabwe" },
];

onMounted(() => {
  if (!user.value || !user.value?.emailVerified) {
    return router.push({ name: "profile" });
  }
});
</script>
