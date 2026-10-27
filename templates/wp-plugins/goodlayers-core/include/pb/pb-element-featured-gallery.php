<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('featured-gallery', 'gdlr_core_pb_element_featured_gallery'); 
	
	if( !class_exists('gdlr_core_pb_element_featured_gallery') ){
		class gdlr_core_pb_element_featured_gallery{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'icon_images',
					'title' => esc_html__('Featured Gallery', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'gallery' => array(
						'title' => esc_html__('Gallery', 'goodlayers-core'),
						'options' => array(
							'gallery' => array(
								'type' => 'custom',
								'item-type' => 'gallery',
								'wrapper-class' => 'gdlr-core-fullsize',
							), 
							'thumbnail-size' => array(
								'title' => esc_html__('Thumbnail Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'thumbnail-size'
							),
							'video-button-url' => array(
								'title' => esc_html__('Video Button URL', 'goodlayers-core'),
								'type' => 'text',
							)
						),
					),
					
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'border-radius' => array(
								'title' => esc_html__('Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom ( Item )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						)
					),
					
					
				);
			}

			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-gallery-<?php echo esc_attr($id); ?>" >
if( document.readyState == 'complete' ){
	jQuery(document).ready(function(){
		jQuery('#gdlr-core-preview-gallery-<?php echo esc_attr($id); ?>').parent().gdlr_core_flexslider().gdlr_core_sly().gdlr_core_isotope();
	});
}else{
	jQuery(window).on('load', function(){
		jQuery('#gdlr-core-preview-gallery-<?php echo esc_attr($id); ?>').parent().gdlr_core_flexslider().gdlr_core_sly().gdlr_core_isotope();
	});
}
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}		
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				$settings['border-radius'] = empty($settings['border-radius'])? '': $settings['border-radius'];

				$ret  = '<div class="gdlr-core-featured-gallery-item gdlr-core-item-pdb gdlr-core-item-pdlr clearfix" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( !empty($settings['gallery']) ){

					$ret .= '<div class="gdlr-core-featured-gallery clearfix" >';

					$count = 0;
					$thumbnail_size = empty($settings['thumbnail-size'])? 'full': $settings['thumbnail-size']; 
					$image_size = sizeof($settings['gallery']);
					$lightbox_group = gdlr_core_image_group_id();
					$lb_images = '';

					$ret .= '<div class="gdlr-core-featured-gallery-images" >';
					foreach( $settings['gallery'] as $image ){ $count++;

						$lb_atts = gdlr_core_get_lightbox_atts(array(
							'url' => gdlr_core_get_image_url($image['id']),
							'caption' => gdlr_core_get_image_info($image['id'], 'caption'),
							'group' => $lightbox_group
						));

						if( $count == 1 ){
							$column_class = ($image_size == 1)? 'gdlr-core-column-60': 'gdlr-core-column-40';
							$ret .= '<div class="gdlr-core-media-image ' . esc_attr($column_class) . '" ' . gdlr_core_esc_style(array(
								'border-radius' => $settings['border-radius']
							)) . ' >';
							$ret .= '<a ' . $lb_atts . ' >';
							$ret .= gdlr_core_get_image($image['id'], $thumbnail_size);
							$ret .= '</a>';
							$ret .= '</div>';
						}else if( $count == 2 ){
							if( $image_size == 2 ){
								$ret .= '<div class="gdlr-core-media-image gdlr-core-bg-image gdlr-core-column-20" ' . gdlr_core_esc_style(array(
									'background-image' => $image['id'],
									'border-radius' => $settings['border-radius']
								)) . ' ><a ' . $lb_atts . ' ></a></div>';
							}else{
								$column_class = ($image_size >= 5)? 'gdlr-core-size-5': 'gdlr-core-size-' . $image_size;
								$ret .= '<div class="gdlr-core-column-20 ' . esc_attr($column_class) . '" >';
								$ret .= '<div class="gdlr-core-bg-image" ' . gdlr_core_esc_style(array(
									'background-image' => $image['id'],
									'border-radius' => $settings['border-radius']
								)) . ' ><a ' . $lb_atts . ' ></a></div>';
							}
						}else if( $count == 3 ){
							$ret .= '<div class="gdlr-core-bg-image" ' . gdlr_core_esc_style(array(
								'background-image' => $image['id'],
								'border-radius' => $settings['border-radius']
							)) . ' ><a ' . $lb_atts . ' ></a></div>';
						}else if( ($count == 4 && $image_size > 4) || $count == 5 ){
							$ret .= '<div class="gdlr-core-bg-image" ' . gdlr_core_esc_style(array(
								'background-image' => $image['id'],
								'border-radius' => $settings['border-radius']
							)) . ' ><a ' . $lb_atts . ' ></a></div>';
						}

						if( ($count > 2 && $count < 5 && $count == $image_size) || $count == 5 ){
							$ret .= '</div>';
						}

						if( ($count == 4 && $image_size == 4) || $count > 5 ){
							$lb_images .= '<a ' . $lb_atts . ' ></a>';
						}
					}
					$ret .= '</div>';
					
					$ret .= '<div class="gdlr-core-featured-gallery-button-wrap" >';
					if( !empty($settings['gallery']) ){
						$ret .= '<span class="gdlr-core-featured-gallery-button gdlr-core-gallery" >';
						$ret .= '<svg width="18" height="18" viewBox="0 0 18 18" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M16.875 11.5803C17.0491 11.5803 17.216 11.5116 17.339 11.3894C17.4621 11.2672 17.5313 11.1014 17.5313 10.9286V2.48246C17.5311 1.96394 17.3237 1.46669 16.9545 1.10004C16.5853 0.733382 16.0846 0.527337 15.5625 0.527199H2.43751C2.17887 0.526472 1.92265 0.576729 1.68371 0.675058C1.44477 0.773388 1.22786 0.917834 1.04554 1.10003C0.677519 1.46746 0.470264 1.96421 0.468757 2.48246V11.607C0.469177 11.7358 0.507943 11.8616 0.580175 11.9685C0.652406 12.0754 0.754876 12.1587 0.874684 12.208C0.994491 12.2572 1.12628 12.2701 1.25347 12.2451C1.38065 12.2201 1.49755 12.1583 1.58944 12.0674L5.06251 8.61815L9.84854 13.3713C9.97231 13.4901 10.1381 13.5558 10.3101 13.5543C10.4822 13.5528 10.6468 13.4842 10.7685 13.3634C10.8902 13.2426 10.9592 13.0791 10.9607 12.9082C10.9622 12.7373 10.896 12.5727 10.7765 12.4498L9.92801 11.607L11.625 9.92166L16.2188 14.484V15.5176C16.2188 15.6904 16.1496 15.8562 16.0265 15.9784C15.9035 16.1007 15.7366 16.1693 15.5625 16.1693H2.43751C2.26346 16.1693 2.09654 16.1007 1.97347 15.9784C1.8504 15.8562 1.78126 15.6904 1.78126 15.5176V14.2141C1.78126 14.0412 1.71212 13.8754 1.58905 13.7532C1.46598 13.631 1.29906 13.5623 1.12501 13.5623C0.950959 13.5623 0.784039 13.631 0.660968 13.7532C0.537898 13.8754 0.468757 14.0412 0.468757 14.2141V15.5176C0.468054 15.7744 0.518671 16.0289 0.617677 16.2662C0.716683 16.5035 0.86211 16.7189 1.04554 16.9C1.4155 17.2655 1.91568 17.4713 2.43751 17.4728H15.5625C16.0846 17.4727 16.5853 17.2667 16.9545 16.9C17.3237 16.5333 17.5311 16.0361 17.5313 15.5176V14.2141C17.5303 14.0411 17.4611 13.8755 17.3385 13.7527L12.089 8.53923C12.0281 8.47868 11.9557 8.43065 11.8761 8.39788C11.7965 8.36511 11.7112 8.34824 11.625 8.34824C11.5388 8.34824 11.4535 8.36511 11.3739 8.39788C11.2943 8.43065 11.2219 8.47868 11.161 8.53923L9.00001 10.6854L5.52648 7.23572C5.46557 7.17517 5.39324 7.12714 5.31363 7.09437C5.23402 7.0616 5.14869 7.04473 5.06251 7.04473C4.97633 7.04473 4.891 7.0616 4.81139 7.09437C4.73178 7.12714 4.65945 7.17517 4.59854 7.23572L1.78126 10.0336V2.48246C1.78126 2.30961 1.8504 2.14383 1.97347 2.0216C2.09654 1.89938 2.26346 1.83071 2.43751 1.83071H15.5625C15.7366 1.83071 15.9035 1.89938 16.0265 2.0216C16.1496 2.14383 16.2188 2.30961 16.2188 2.48246V10.9286C16.2188 11.1014 16.2879 11.2672 16.411 11.3894C16.534 11.5116 16.701 11.5803 16.875 11.5803Z" fill="black"/></svg>';
						$ret .= esc_html__('Gallery', 'goodlayers-core') . '</span>';
					}
					
					if( !empty($settings['video-button-url']) ){
						$lb_atts = gdlr_core_get_lightbox_atts(array(
							'class' => 'gdlr-core-featured-gallery-button gdlr-core-video',
							'url' => $settings['video-button-url']
						));
						$ret .= '<a ' . $lb_atts . ' >';
						$ret .= '<svg width="20" height="14" viewBox="0 0 20 14" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M19.427 1.84835C19.3273 1.77904 19.2124 1.73498 19.0922 1.7199C18.9719 1.70483 18.8499 1.71917 18.7363 1.76173L13.75 3.64445V2.46895C13.7493 1.8683 13.5121 1.29244 13.0903 0.867713C12.6685 0.442989 12.0965 0.20409 11.5 0.20343H2.5C1.90346 0.20409 1.33155 0.442989 0.909733 0.867713C0.487918 1.29244 0.250655 1.8683 0.25 2.46895V11.531C0.251723 12.1315 0.488585 12.7071 0.909175 13.1328C1.11756 13.3439 1.36547 13.5112 1.63854 13.6252C1.91161 13.7391 2.20442 13.7973 2.5 13.7965H9.57153C9.77044 13.7965 9.9612 13.717 10.1019 13.5753C10.2425 13.4337 10.3215 13.2416 10.3215 13.0414C10.3215 12.8411 10.2425 12.649 10.1019 12.5074C9.9612 12.3657 9.77044 12.2862 9.57153 12.2862H2.5C2.30109 12.2862 2.11032 12.2066 1.96967 12.065C1.82902 11.9234 1.75 11.7313 1.75 11.531V2.46895C1.75018 2.26872 1.82925 2.07674 1.96987 1.93516C2.11048 1.79358 2.30114 1.71395 2.5 1.71377H11.5C11.6989 1.71395 11.8895 1.79358 12.0301 1.93516C12.1707 2.07674 12.2498 2.26872 12.25 2.46895V13.0414C12.25 13.2416 12.329 13.4337 12.4697 13.5753C12.6103 13.717 12.8011 13.7965 13 13.7965C13.1989 13.7965 13.3897 13.717 13.5303 13.5753C13.671 13.4337 13.75 13.2416 13.75 13.0414V10.3555L18.7363 12.2382C18.8207 12.2699 18.91 12.2861 19 12.2862C19.1989 12.286 19.3896 12.2064 19.5302 12.0648C19.6708 11.9233 19.7499 11.7313 19.75 11.531V2.46895C19.75 2.34691 19.7206 2.2267 19.6643 2.11862C19.6081 2.01053 19.5266 1.91779 19.427 1.84835V1.84835ZM18.25 10.4414L13.75 8.74224V5.25803L18.25 3.55889V10.4414Z" fill="black"/></svg>';
						$ret .= esc_html__('Video', 'goodlayers-core') . '</a>';
					}
					$ret .= $lb_images;
					$ret .= '</div>';

					$ret .= '</div>';

				}else{
					$ret .= '<div class="gdlr-core-external-plugin-message">' . esc_html__('No image available. Please edit this item to select images.', 'goodlayers-core') . '</div>';
				}

				$ret .= '</div>'; // gdlr-core-gallery-item
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_featured_gallery
	} // class_exists	
